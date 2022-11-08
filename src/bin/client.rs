use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
pub async fn main() -> Result<()> {
    // Open a TCP stream to the socket address.
    //
    // Note that this is the Tokio TcpStream, which is fully async.
    tcp_tunnel::connect("127.0.0.1:6666", "hello").await?;
    println!("created stream");
    Ok(())
}
