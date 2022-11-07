// pub write<T>(steam: )

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum Action {
    Bind(String)

}


pub async fn write<W>(mut writer: W, data: &[u8]) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin + Send + 'static,
{
    let len = data.len();
    writer.write_u16(len as u16).await?;
    writer.write_all(data).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn read<R>(mut reader: R) -> std::io::Result<Vec<u8>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let len = reader.read_u16().await?;
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_write() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut client, mut server) = tokio::io::duplex(1024);

        super::write(server, b"pong").await.unwrap();

        let msg = super::read(client).await.unwrap();
        let msg = String::from_utf8_lossy(&msg);
        assert_eq!("pong", msg)
    }
}
