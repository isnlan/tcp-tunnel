use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
pub async fn main() -> Result<()> {
    // Open a TCP stream to the socket address.
    //
    // Note that this is the Tokio TcpStream, which is fully async.
    let mut stream = TcpStream::connect("127.0.0.1:6666").await?;
    println!("created stream");


    tcp_tunnel::pack::write_all(&mut stream, b"hello world\n").await?;
    // let result = stream.write_all(b"hello world\n").await;
    // println!("wrote to stream; success={:?}", result.is_ok());

    Ok(())
}