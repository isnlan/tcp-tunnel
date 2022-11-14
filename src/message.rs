use crate::utils;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Connect {
    pub id: i64,
    pub conn_id: i64,
    pub proto: String,
    pub addr: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Data {
    pub id: i64,
    pub conn_id: i64,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Message {
    Token(String),
    Connect(Connect),
    Data(Data),
    Close,
}

impl Message {
    pub async fn read<R: AsyncRead + Unpin>(stream: &mut R) -> Result<Message> {
        let data = utils::read(stream).await?;
        bincode::deserialize(&data).map_err(|err| anyhow!(err))
    }

    pub async fn write<W: AsyncWrite + Unpin>(&self, stream: &mut W) -> Result<()> {
        let encoded: Vec<u8> = bincode::serialize(self)?;
        utils::write(stream, &encoded).await?;
        Ok(())
    }
}
