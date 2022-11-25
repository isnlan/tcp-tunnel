use crate::session::Session;
use anyhow::{Context, Result};
use backoff::future::retry_notify;
use backoff::ExponentialBackoff;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use tokio::time::Duration;
use tracing::warn;
use crate::message::Message;

pub async fn connect(addr: &str, token: &str) -> Result<()> {
    let backoff = ExponentialBackoff {
        max_interval: Duration::from_millis(100),
        max_elapsed_time: Some(Duration::from_secs(10)),
        ..Default::default()
    };

    let mut stream: TcpStream = retry_notify(
        backoff,
        || async {
            TcpStream::connect(addr)
                .await
                .with_context(|| format!("Failed to connect to {}", addr))
                .map_err(backoff::Error::transient)
        },
        |e, duration| {
            warn!("{:#}. Retry in {:?}", e, duration);
        },
    )
    .await?;

    let msg = Message::Token(token.to_string());
    msg.write(&mut stream).await?;

    // tokio::time::sleep(Duration::from_secs(100)).await;
    let session: Session = Session::new(token.to_string(), rand::random(), stream, true);

    // task::spawn(async move {
    let _ = session.serve().await;
    // });

    Ok(())
}
