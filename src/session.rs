use crate::{Connect, Message, MyStream};
use anyhow::Result;

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

pub struct Session {
    next_conn_id: AtomicI64,
    token: String,
    session_key: i64,
    stream: Mutex<TcpStream>,
    conns: HashMap<i64, MyStream>,
    client: bool,
    msg_tx: Sender<Message>,
    msg_rx: Mutex<Receiver<Message>>,
}

impl Session {
    pub fn new(token: String, session_key: i64, steam: TcpStream, client: bool) -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            next_conn_id: AtomicI64::new(1),
            token,
            session_key,
            stream: Mutex::new(steam),
            conns: HashMap::new(),
            client,
            msg_tx: tx,
            msg_rx: Mutex::new(rx),
        }
    }

    pub async fn serve(&self) -> Result<()> {
        let mut stream = self.stream.lock().await;
        let (mut read, mut write) = stream.split();

        let r = async {
            if let Err(err) = self.process_read(&mut read).await {
                error!("session process read error: {:?}", err);
            }

            self.msg_tx.send(Message::Close).await
        };

        let w = async {
            if let Err(err) = self.process_write(&mut write).await {
                error!("session process write error: {:?}", err);
            }
        };

        tokio::join!(r, w);

        warn!("session server closed!");

        Ok(())
    }

    pub async fn get_stream(&self, proto: &str, addr: &str) -> Result<MyStream> {
        let conn_id = self.next_conn_id.fetch_add(1, Ordering::SeqCst);
        let stream = MyStream::new(conn_id, proto, addr);

        let msg = Message::Connect(Connect {
            id: 0,
            conn_id,
            proto: proto.to_string(),
            addr: proto.to_string(),
        });

        self.msg_tx.send(msg).await?;

        Ok(stream)
    }

    async fn process_read<R: AsyncReadExt + Unpin>(&self, read: &mut R) -> Result<()> {
        loop {
            let msg = Message::read(read).await?;

            match msg {
                Message::Connect(connect) => {
                    self.client_connect(connect).await?;
                    // info!("create connect!! -> {:?}", connect);
                    // tokio::io::copy_bidirectional(a, b)
                }
                _ => return Ok(()),
            }
        }
    }

    async fn process_write<R: AsyncWriteExt + Unpin>(&self, write: &mut R) -> Result<()> {
        let mut rx = self.msg_rx.lock().await;
        while let Some(msg) = rx.recv().await {
            // todo break loop
            match msg {
                Message::Close => break,
                _ => {
                    debug!("send -> {:?}", msg);

                    msg.write(write).await?;
                }
            }
        }
        Ok(())
    }

    async fn client_connect(&self, connect: Connect) -> Result<()> {
        let mut mystream = MyStream::new(connect.conn_id, &connect.proto, &connect.addr);
        if connect.proto == "tcp" {
            let mut stream = TcpStream::connect(connect.addr).await?;

            tokio::io::copy_bidirectional(&mut mystream, &mut stream).await?;
        }
        Ok(())
    }
}
