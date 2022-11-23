use std::{io::ErrorKind, pin::Pin, sync::Mutex, task::Poll};

use futures_util::pin_mut;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream, ReadBuf},
    sync::mpsc::Sender,
};
use tracing::error;

use crate::{Data, Message};

pub struct Stream {
    conn_id: i64,
    reader: Mutex<DuplexStream>,
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
    writer: Mutex<DuplexStream>,
}

impl StreamStub {
    pub async fn on_message(&self, data: Data) -> anyhow::Result<()> {
        self.writer.lock().unwrap().write(&data.data).await?;

        Ok(())
    }
}

pub fn new(
    conn_id: i64,
    proto: &str,
    addr: &str,
    msg_bus: Sender<Message>,
) -> (Stream, StreamStub) {
    let (reader, writer) = tokio::io::duplex(1024);
    (
        Stream {
            conn_id,
            reader: Mutex::new(reader),
            msg_bus,
        },
        StreamStub {
            conn_id,
            proto: proto.to_string(),
            addr: addr.to_string(),
            writer: Mutex::new(writer),
        },
    )
}

impl AsyncRead for Stream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Pin::new(self.reader).poll().read(cx, buf)
        let mut v = vec![];
        let mut reader = self.reader.lock().unwrap();
        // let f0 = reader.read(&mut v);
        let f1 = futures_util::future::maybe_done(reader.read(&mut v));
        pin_mut!(f1);
        match f1.as_mut().take_output() {
            Some(ret) => match ret {
                Ok(n) => {
                    buf.put_slice(&v);
                    Poll::Ready(Ok(()))
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            None => Poll::Pending,
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        todo!()
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        todo!()
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        todo!()
    }
}
