use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn write_all<W: AsyncWriteExt + Unpin>(stream: &mut W, data: &[u8]) -> io::Result<()> {
    let size = data.len();

    stream.write_u32(size as u32).await?;
    stream.write_all(data).await?;
    Ok(())
}

pub async fn read_data<R: AsyncReadExt + Unpin>(stream: &mut R) -> io::Result<Vec<u8>> {
    let size = stream.read_u32().await?;
    let mut data = vec![0; size as usize];

    // 读取数据body
    stream.read_exact(&mut data[..]).await?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Cursor};

    use super::*;

    #[tokio::test]
    async fn it_works() -> anyhow::Result<()> {
        let mut writer = Vec::new();

        let s = b"hello world\n";
        write_all(&mut writer, s).await?;

        let mut reader = Cursor::new(writer);
        let s1 = read_data(&mut reader).await?;

        assert_eq!(s1.as_slice(), s);

        Ok(())
    }
}
