use anyhow::Result;
use tokio::task;

use crate::message::{Connect, Message};
use crate::stream::{Stream, StreamStub};
use crate::{message, stream};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

pub struct Session {
    next_conn_id: AtomicI64,
    token: String,
    session_key: i64,
    stream: Mutex<TcpStream>,
    conns: Mutex<HashMap<i64, StreamStub>>,
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
            conns: Mutex::new(HashMap::new()),
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

            self.msg_tx.send(message::Message::Close).await
        };

        let w = async {
            if let Err(err) = self.process_write(&mut write).await {
                error!("session process write error: {:?}", err);
            }
        };

        let _ = tokio::join!(r, w);

        warn!("session server closed!");

        Ok(())
    }

    pub async fn get_stream(&self, proto: &str, addr: &str) -> Result<Stream> {
        let conn_id = self.next_conn_id.fetch_add(1, Ordering::SeqCst);

        let (stream, stub) = stream::new(conn_id, proto, addr, self.msg_tx.clone());

        let mut conns = self.conns.lock().await;
        conns.insert(conn_id, stub);

        let msg = Message::Connect(Connect {
            id: 0,
            conn_id,
            proto: proto.to_string(),
            addr: addr.to_string(),
        });

        self.msg_tx.send(msg).await?;

        Ok(stream)
    }

    async fn process_read<R: AsyncRead + Unpin>(&self, read: &mut R) -> Result<()> {
        loop {
            let msg = Message::read(read).await?;

            debug!("msg: {:?}", msg);

            match msg {
                Message::Connect(connect) => {
                    self.client_connect(connect).await?;
                }
                Message::Data(data) => {
                    let conns = self.conns.lock().await;

                    match conns.get(&data.conn_id) {
                        Some(stub) => stub.on_message(data).await.unwrap(),

                        None => {
                            continue;
                        }
                    }

                    // return Ok(());
                }

                _ => return Ok(()),
            }
        }
    }

    async fn process_write<R: AsyncWrite + Unpin>(&self, write: &mut R) -> Result<()> {
        let mut rx = self.msg_rx.lock().await;
        while let Some(msg) = rx.recv().await {
            // todo break loop
            match msg {
                Message::Close => break,
                _ => {
                    debug!("send -> {:?}", msg);

                    msg.write(write).await;
                }
            }
        }
        Ok(())
    }

    async fn client_connect(&self, connect: Connect) -> Result<()> {
        if connect.proto == "tcp" {
            let mut stream = TcpStream::connect(&connect.addr).await?;

            let (mut s, stub) = stream::new(
                connect.conn_id,
                &connect.proto,
                &connect.addr,
                self.msg_tx.clone(),
            );

            task::spawn(async move {
                tokio::io::copy_bidirectional(&mut stream, &mut s)
                    .await
                    .unwrap();
            });

            let mut conns = self.conns.lock().await;
            conns.insert(connect.conn_id, stub);
        }

        Ok(())
    }
}
