use crate::{utils, Message, Connect};
use anyhow::{anyhow, bail, Context, Result};
use backoff::ExponentialBackoff;
use backoff::{backoff::Backoff, future::retry_notify};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{self, Duration, Instant};
use tracing::{debug, error, info, instrument, trace, warn, Instrument, Span};
use crate::session::Session;

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


    let msg = Message::Connect(Connect{
        id: 12,
        connect_id: 12,
        proto: "tcp".to_string(),
        address: "127.0.0.1".to_string()
    });
    msg.write(&mut stream).await?;

    tokio::time::sleep(Duration::from_secs(100)).await;
    // let session = Session::new(token.to_string(), rand::random(), stream, true);

    Ok(())
}
