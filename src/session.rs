use crate::Message;
use anyhow::Result;
use std::collections::HashMap;
use std::ptr::read;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task;

pub struct Session {
    next_conn_id: i64,
    token: String,
    session_key: i64,
    stream: TcpStream,
    conns: HashMap<i64, Connection>,
    client: bool,
}

impl Session {
    pub fn new(token: String, session_key: i64, stream: TcpStream, client: bool) -> Self {
        Self {
            next_conn_id: 1,
            token,
            session_key,
            stream,
            conns: HashMap::new(),
            client,
        }
    }

    pub async fn serve(&mut self) -> Result<()> {
        let (mut read, mut write) = self.stream.split();

        let r = async {
            let _ = process_tunnel_read(&mut read).await;
        };

        let w = async {
            let _ = process_tunnel_write(&mut write).await;
        };

        tokio::join!(r, w);

        Ok(())
    }

    fn serve_message(&self, msg: Message) -> Result<()> {
        Ok(())
    }
}

async fn process_tunnel_read<R:  AsyncReadExt + Unpin>(read: &mut R) -> Result<()> {
    loop {
        let msg = Message::read(read).await?;

        match msg {
            Message::Connect(connect)=> {
                println!("create connect!!");
            }
            _ => {
                return Ok(())
            }
        }
    }
}
async fn process_tunnel_write<R:  AsyncWriteExt + Unpin>(write: &mut R) -> Result<()> {
   Ok(())
}

pub struct Connection {}
