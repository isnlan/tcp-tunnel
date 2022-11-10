use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:6666";
    let addr = addr.parse::<SocketAddr>()?;
    let server = Arc::new(tcp_tunnel::Server::new(MyAuth {}, addr));

    let server_c = server.clone();
    task::spawn(async move {
        println!("--run--");
        server_c.serve().await.unwrap();
    });

    tokio::time::sleep(Duration::from_secs(5)).await;

    let stream = server.get_stream("hello", "tcp", "127.0.0.1:80").await?;

    println!("--> {:?}", stream.unwrap());

    Ok(())
}

struct MyAuth {}

impl tcp_tunnel::Authorizer for MyAuth {
    fn auth(&self, token: &str) -> bool {
        println!("token: {}", token);

        true
    }
}
