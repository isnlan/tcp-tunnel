use anyhow::Result;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:6666";
    let addr = addr.parse::<SocketAddr>()?;
    let server = tcp_tunnel::Server::new(MyAuth {}, addr);

    server.serve().await?;

    Ok(())
}

struct MyAuth {}

impl tcp_tunnel::Authorizer for MyAuth {
    fn auth(&self, token: &str) -> bool {
        println!("token: {}", token);

        true
    }
}
