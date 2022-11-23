use futures_util::{pin_mut, FutureExt};
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};

use bytes::{Buf, BytesMut};
use std::io::ErrorKind;
use std::{
    pin::Pin,
    sync::Arc,
    task::{self, Poll, Waker},
};

use tokio::sync::mpsc::Sender;
use tracing::error;

// use crossbeam_channel::Sender;

// use tokio::sync::Mutex;
use crate::mutex::Mutex;
use crate::{Data, Message};

/// A unidirectional IO over a piece of memory.
///
/// Data can be written to the pipe, and reading will return that data.
#[derive(Debug)]
struct Pipe {
    /// The buffer storing the bytes written, also read from.
    ///
    /// Using a `BytesMut` because it has efficient `Buf` and `BufMut`
    /// functionality already. Additionally, it can try to copy data in the
    /// same buffer if there read index has advanced far enough.
    buffer: BytesMut,
    /// Determines if the write side has been closed.
    is_closed: bool,
    /// The maximum amount of bytes that can be written before returning
    /// `Poll::Pending`.
    max_buf_size: usize,
    /// If the `read` side has been polled and is pending, this is the waker
    /// for that parked task.
    read_waker: Option<Waker>,
    /// If the `write` side has filled the `max_buf_size` and returned
    /// `Poll::Pending`, this is the waker for that parked task.
    write_waker: Option<Waker>,
}

impl Pipe {
    fn new(max_buf_size: usize) -> Self {
        Pipe {
            buffer: BytesMut::new(),
            is_closed: false,
            max_buf_size,
            read_waker: None,
            write_waker: None,
        }
    }

    fn close_write(&mut self) {
        self.is_closed = true;
        // needs to notify any readers that no more data will come
        if let Some(waker) = self.read_waker.take() {
            waker.wake();
        }
    }

    fn close_read(&mut self) {
        self.is_closed = true;
        // needs to notify any writers that they have to abort
        if let Some(waker) = self.write_waker.take() {
            waker.wake();
        }
    }

    fn poll_read_internal(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.buffer.has_remaining() {
            let max = self.buffer.remaining().min(buf.remaining());
            buf.put_slice(&self.buffer[..max]);
            self.buffer.advance(max);
            if max > 0 {
                // The passed `buf` might have been empty, don't wake up if
                // no bytes have been moved.
                if let Some(waker) = self.write_waker.take() {
                    waker.wake();
                }
            }
            Poll::Ready(Ok(()))
        } else if self.is_closed {
            Poll::Ready(Ok(()))
        } else {
            self.read_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }

    fn poll_write_internal(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.is_closed {
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }
        let avail = self.max_buf_size - self.buffer.len();
        if avail == 0 {
            self.write_waker = Some(cx.waker().clone());
            return Poll::Pending;
        }

        let len = buf.len().min(avail);
        self.buffer.extend_from_slice(&buf[..len]);
        if let Some(waker) = self.read_waker.take() {
            waker.wake();
        }
        Poll::Ready(Ok(len))
    }
}

impl AsyncRead for Pipe {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.poll_read_internal(cx, buf)
    }
}

impl AsyncWrite for Pipe {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.poll_write_internal(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        _: &mut task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.close_write();
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
// #[pin_project]
pub struct Stream {
    conn_id: i64,
    pipe: Arc<Mutex<Pipe>>,
    // #[pin]
    msg_bus: Sender<Message>,
}

impl Stream {
    async fn write_internal(&self, buf: &[u8]) -> std::io::Result<usize> {
        let data = Data {
            id: 12,
            conn_id: self.conn_id,
            data: buf.to_vec(),
        };

        match self.msg_bus.send(Message::Data(data)).await {
            Ok(_) => Ok(buf.len()),
            Err(err) => {
                error!("send error: {}", err);
                Err(std::io::Error::new(ErrorKind::BrokenPipe, "send error"))
            }
        }
    }
}

pub struct StreamStub {
    conn_id: i64,
    proto: String,
    addr: String,
    pipe: Arc<Mutex<Pipe>>,
}

impl StreamStub {
    pub async fn on_message(&self, data: Data) -> anyhow::Result<()> {
        // self.tx.send(msg).await?;
        let mut pipe = self.pipe.lock();
        pipe.write(&data.data).await?;

        Ok(())
    }
}

pub fn new(
    conn_id: i64,
    proto: &str,
    addr: &str,
    msg_bus: Sender<Message>,
) -> (Stream, StreamStub) {
    let buffer = Arc::new(Mutex::new(Pipe::new(1024)));

    (
        Stream {
            conn_id,
            pipe: buffer.clone(),
            msg_bus,
        },
        StreamStub {
            conn_id,
            proto: proto.to_string(),
            addr: addr.to_string(),
            pipe: buffer,
        },
    )
}

impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.pipe.lock()).poll_read(cx, buf)
    }
}

impl AsyncWrite for Stream {
    #[allow(unused_mut)]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        // let this = self.project();
        // this.write_internal(buf).poll(cx)

        let data = Data {
            id: 12,
            conn_id: self.conn_id,
            data: buf.to_vec(),
        };

        let f1 = futures_util::future::maybe_done(self.write_internal(buf));
        pin_mut!(f1);
        match f1.as_mut().take_output() {
            Some(ret) => match ret {
                Ok(n) => Poll::Ready(Ok(n)),
                Err(_) => todo!(),
            },
            None => Poll::Pending,
        }
    }

    #[allow(unused_mut)]
    fn poll_flush(
        mut self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    #[allow(unused_mut)]
    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.pipe.lock()).poll_shutdown(cx)
    }
}
