use std::{future::Future, io::ErrorKind};

// use futures_util::pin_mut;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream, ReadBuf},
    sync::{mpsc::Sender, Mutex},
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
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Box::pin(self.read_internal(buf)).as_mut().poll(cx)
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        todo!()
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        todo!()
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        todo!()
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
    async fn it_works() -> anyhow::Result<()> {
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

    #[test]
    fn test_reader() {
        let mut v = [0u8; 10];
        let mut buf = tokio::io::ReadBuf::new(&mut v);
        println!("remaing {}", buf.remaining());
        buf.put_slice("ping".as_bytes());

        println!("remaing {}", buf.remaining());
        // println!(" --- >{:?}", v);
        let mut b1 = buf.filled();
        // b1.(&b"fsf"[..]);
        println!("0 --- >{:?}", b1);

        println!("1 --- >{:?}", v);
        // println!("1 --- >{:?}", b1);
    }

    #[tokio::test]
    async fn test_readr_trait() -> anyhow::Result<()> {
        let (mut a, mut b) = tokio::io::duplex(8);
        let l1 = a.write(b"pingpingping").await?;

        let mut v = [0u8; 12];
        let mut buf = tokio::io::ReadBuf::new(&mut v);
        // buf.put_slice("hello".as_bytes());

        let b1 =
            unsafe { &mut *(buf.unfilled_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8]) };

        let ret = b.read(b1).await?;
        unsafe { buf.assume_init(ret) };
        buf.advance(ret);

        println!("========== {:?}", v);

        Ok(())
    }

    #[tokio::test]
    async fn test_readr_trait1() -> anyhow::Result<()> {
        let (mut a, mut b) = tokio::io::duplex(8);

        task::spawn(async move {
            let l1 = a.write_all(b"pingpingping").await.unwrap();
        });

        println!("-----");
        for i in 0..3 {
            let mut v = [0u8; 10];
            let mut buf = tokio::io::ReadBuf::new(&mut v);
            // buf.put_slice("hello".as_bytes());

            let b1 = unsafe {
                &mut *(buf.unfilled_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8])
            };

            let ret = b.read_exact(b1).await?;
            unsafe { buf.assume_init(ret) };
            buf.advance(ret);

            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("========== {:?}", v);
        }

        Ok(())
    }
}
