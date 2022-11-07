use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const TCP_DATA_LEN: usize = 4;

pub async fn write_all(stream: &mut TcpStream, data: &[u8]) -> io::Result<()> {
    let len = data.len();

    let mut header = [0u8; TCP_DATA_LEN];

    header[0] = (len >> 24) as u8;
    header[1] = (len >> 16) as u8;
    header[2] = (len >> 8) as u8;
    header[3] = len as u8;

    stream.write_all(&header).await?;
    stream.write_all(data).await?;
    Ok(())
}

pub async fn read_data(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut header = [0u8; TCP_DATA_LEN];
    stream.read_exact(&mut header).await?;

    let size = get_len(&header);

    let mut data = vec![0; size as usize];

    // 读取数据body
    stream.read_exact(&mut data[..]).await?;

    Ok(data)
}

#[inline]
fn get_len(header: &[u8]) -> usize {
    ((header[0] as usize) << 24)
        | ((header[1] as usize) << 16)
        | ((header[2] as usize) << 8)
        | (header[3] as usize)
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;

    use super::*;

    #[tokio::test]
    async fn it_works() {

    }
}
