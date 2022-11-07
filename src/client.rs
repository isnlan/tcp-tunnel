use anyhow::{anyhow, bail, Context, Result};
use backoff::ExponentialBackoff;
use backoff::{backoff::Backoff, future::retry_notify};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{self, Duration, Instant};
use tracing::{debug, error, info, instrument, trace, warn, Instrument, Span};

pub async fn connect(addr: &str, token: &Vec<u8>) -> Result<()> {
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

    Ok(())
}
