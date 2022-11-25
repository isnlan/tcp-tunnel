use std::task::Poll;
use std::{future::Future, io::ErrorKind};

// use futures_util::pin_mut;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream, ReadBuf},
    sync::{mpsc::Sender, Mutex},
};
use tracing::error;

use crate::message::{Data, Message};

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

    async fn read_internal(&self, buf: &mut ReadBuf<'_>) -> std::io::Result<()> {
        let b =
            unsafe { &mut *(buf.unfilled_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8]) };

        let mut reader = self.reader.lock().await;

        let ret = reader.read(b).await?;
        unsafe { buf.assume_init(ret) };
        buf.advance(ret);

        Ok(())
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
        self.writer.lock().await.write_all(&data.data).await?;

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
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Box::pin(self.read_internal(buf)).as_mut().poll(cx)
    }
}

impl AsyncWrite for Stream {
    #[allow(unused_mut)]
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Box::pin(self.write_internal(buf)).as_mut().poll(cx)
    }

    #[allow(unused_mut)]
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    #[allow(unused_mut)]
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Ok;
    use bytes::BufMut;
    use tokio::{sync, task};

    use super::*;

    #[tokio::test]
    async fn test_reader() -> anyhow::Result<()> {
        let (bus_tx, _bus_rx) = sync::mpsc::channel(10);

        let (mut a, b) = new(1, "tcp", "127.0.0.1:8888", bus_tx);

        task::spawn(async move {
            b.on_message(Data {
                id: 1,
                conn_id: 1,
                data: "ping".as_bytes().to_vec(),
            })
            .await
            .unwrap();
        })
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut buf = [0u8; 40];

        // let mut rbf = ReadBuf::new(&mut buf);
        // let v1 = a.read_internel(&mut rbf).await?;

        let _len = a
            .read(&mut buf)
            .await
            .map_err(|e| {
                println! {"err {:?}", e};
                e
            })
            .unwrap();
        println!("-- {:?}", buf);
        // assert_eq!(&buf, b"ping");

        Ok(())
    }

    #[tokio::test]
    async fn test_writer() -> anyhow::Result<()> {
        let (bus_tx, mut bus_rx) = sync::mpsc::channel(10);

        let (mut a, b) = new(1, "tcp", "127.0.0.1:8888", bus_tx);

        task::spawn(async move {
            while let Some(msg) = bus_rx.recv().await {
                println!("<<-- {:?}", msg);
            }
        });

        a.write(b"hello").await?;
        tokio::time::sleep(Duration::from_secs(1)).await;

        Ok(())
    }
}
