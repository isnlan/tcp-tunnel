use std::sync::Arc;

use crate::{Data, Message};

use anyhow::Result;
use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Stream {
    conn_id: i64,
    buffer: Arc<Mutex<BytesMut>>,
    msg_bus: Sender<Message>,
}

pub struct StreamStub {
    conn_id: i64,
    proto: String,
    addr: String,
    buffer: Arc<Mutex<BytesMut>>,
}

impl StreamStub {
    pub async fn on_message(&self, data: Data) -> Result<()> {
        // self.tx.send(msg).await?;
        let mut lock = self.buffer.lock().await;
        lock.extend_from_slice(&data.data);

        Ok(())
    }
}

pub fn new(
    conn_id: i64,
    proto: &str,
    addr: &str,
    msg_bus: Sender<Message>,
) -> (Stream, StreamStub) {
    let buffer = Arc::new(Mutex::new(BytesMut::new()));

    (
        Stream {
            conn_id,
            msg_bus,
            buffer: buffer.clone(),
        },
        StreamStub {
            conn_id,
            proto: proto.to_string(),
            addr: addr.to_string(),
            buffer,
        },
    )
}

impl AsyncRead for Stream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        todo!()
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
