use crate::{Connect, Message, MyStream};
use anyhow::Result;

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct Session {
    next_conn_id: AtomicI64,
    token: String,
    session_key: i64,
    stream: Mutex<TcpStream>,
    conns: HashMap<i64, MyStream>,
    client: bool,
}

impl Session {
    pub fn new(token: String, session_key: i64, steam: TcpStream, client: bool) -> Self {
        Self {
            next_conn_id: AtomicI64::new(1),
            token,
            session_key,
            stream: Mutex::new(steam),
            conns: HashMap::new(),
            client,
        }
    }

    pub async fn serve(&self) -> Result<()> {
        let mut stream = self.stream.lock().await;
        let (mut read, mut write) = stream.split();

        let r = async {
            let _ = self.process_read(&mut read).await;
        };

        let w = async {
            let _ = self.process_write(&mut write).await;
        };

        tokio::join!(r, w);

        Ok(())
    }

    pub fn get_stream(&self, proto: &str, addr: &str) -> Result<MyStream> {
        let conn_id = self.next_conn_id.fetch_add(1, Ordering::SeqCst);
        let stream = MyStream::new(conn_id, proto, addr);

        let _msg = Message::Connect(Connect {
            id: 0,
            conn_id,
            proto: proto.to_string(),
            addr: proto.to_string(),
        });
        Ok(stream)
    }

    async fn process_read<R: AsyncReadExt + Unpin>(&self, read: &mut R) -> Result<()> {
        loop {
            let msg = Message::read(read).await?;

            match msg {
                Message::Connect(_connect) => {
                    println!("create connect!!");
                }
                _ => return Ok(()),
            }
        }
    }

    async fn process_write<R: AsyncWriteExt + Unpin>(&self, _write: &mut R) -> Result<()> {
        Ok(())
    }
}

pub struct Connection {}
