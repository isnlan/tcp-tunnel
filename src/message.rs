use crate::utils;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Connect {
    pub id: i64,
    pub connect_id: i64,
    pub proto: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Data {
    id: i64,
    connect_id: i64,
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Message {
    Token(String),
    Connect(Connect),
    Data(Data),
}

impl Message {
    pub async fn read<R:  AsyncReadExt + Unpin>(stream: &mut R) -> Result<Message> {
        let data = utils::read_data(stream).await?;
        bincode::deserialize(&data).map_err(|err| anyhow!(err))
    }

    pub async fn write(&self, stream: &mut TcpStream) -> Result<()> {
        let encoded: Vec<u8> = bincode::serialize(self)?;
        utils::write_all(stream, &encoded).await?;
        Ok(())
    }
}
