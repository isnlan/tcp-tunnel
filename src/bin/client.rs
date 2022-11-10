use anyhow::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    // Open a TCP stream to the socket address.
    //
    // Note that this is the Tokio TcpStream, which is fully async.
    tcp_tunnel::connect("127.0.0.1:6666", "hello").await?;
    println!("created stream");
    Ok(())
}
