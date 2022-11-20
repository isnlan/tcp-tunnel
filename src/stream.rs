use crate::Data;
use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Debug)]
pub struct MyStream<T: AsyncRead + AsyncWrite> {
    conn_id: i64,
    proto: String,
    addr: String,
    // buf_client: tokio::io::DuplexStream,
    buf_server: T,
}

impl<T: AsyncRead + AsyncWrite> MyStream<T> {
    pub fn new(conn_id: i64, proto: &str, addr: &str, buf_server: T) -> Self {
        // let (buf_client, mut buf_server) = tokio::io::duplex(64);
        MyStream {
            conn_id,
            proto: proto.to_string(),
            addr: addr.to_string(),
            buf_server,
        }
    }

    pub fn on_data(data: Data) -> Result<()> {
        Ok(())
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
