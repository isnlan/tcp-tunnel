use crate::{Data};

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::{Receiver};

#[derive(Debug)]
pub struct MyStream<T: AsyncRead + AsyncWrite> {
    conn_id: i64,
    proto: String,
    addr: String,
    stream: T,
    rx: Receiver<Data>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> MyStream<T> {
    pub fn new(conn_id: i64, proto: &str, addr: &str, stream: T, rx: Receiver<Data>) -> Self {
        MyStream {
            conn_id,
            proto: proto.to_string(),
            addr: addr.to_string(),
            stream,
            rx,
        }
    }

    pub async fn serve(&mut self) {
        while let Some(data) = self.rx.recv().await {
            let _ret = self.stream.write_all(&data.data).await.unwrap();
        }
    }
}

// impl AsyncRead for MyStream {
//     fn poll_read(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &mut tokio::io::ReadBuf<'_>,
//     ) -> std::task::Poll<std::io::Result<()>> {
//         todo!()
//     }
// }

// impl AsyncWrite for MyStream {
//     fn poll_write(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &[u8],
//     ) -> std::task::Poll<Result<usize, std::io::Error>> {
//         todo!()
//     }

//     fn poll_flush(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), std::io::Error>> {
//         todo!()
//     }

//     fn poll_shutdown(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), std::io::Error>> {
//         todo!()
//     }
// }
