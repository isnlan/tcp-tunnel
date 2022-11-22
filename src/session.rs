use crate::{Connect, Data, Message};
use anyhow::Result;
use tokio::task;

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

pub struct Session {
    next_conn_id: AtomicI64,
    token: String,
    session_key: i64,
    stream: Mutex<TcpStream>,
    conns: Mutex<HashMap<i64, Sender<Data>>>,
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

            self.msg_tx.send(Message::Close).await
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

    pub async fn get_stream(&self, proto: &str, addr: &str) -> Result<DuplexStream> {
        let conn_id = self.next_conn_id.fetch_add(1, Ordering::SeqCst);

        let (buf_client, buf_server) = tokio::io::duplex(64);

        let (tx, rx) = mpsc::channel(64);

        let msg_bus = self.msg_tx.clone();
        task::spawn(async move {
            process_stream(conn_id, buf_server, rx, msg_bus)
                .await
                .unwrap();
        });

        let mut conns = self.conns.lock().await;
        conns.insert(conn_id, tx);

        let msg = Message::Connect(Connect {
            id: 0,
            conn_id,
            proto: proto.to_string(),
            addr: addr.to_string(),
        });

        self.msg_tx.send(msg).await?;

        Ok(buf_client)
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
                        Some(tx) => tx.send(data).await.unwrap(),

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

                    msg.write(write).await?;
                }
            }
        }
        Ok(())
    }

    async fn client_connect(&self, connect: Connect) -> Result<()> {
        if connect.proto == "tcp" {
            let stream = TcpStream::connect(&connect.addr).await?;

            let msg_bus = self.msg_tx.clone();

            let (tx, rx) = mpsc::channel(64);
            task::spawn(async move {
                process_stream(connect.conn_id, stream, rx, msg_bus)
                    .await
                    .unwrap();
            });

            let mut conns = self.conns.lock().await;
            conns.insert(connect.conn_id, tx);

            // tokio::io::copy_bidirectional(a, b)
        }

        Ok(())
    }
}

async fn process_stream<T: AsyncRead + AsyncWrite + Unpin>(
    conn_id: i64,
    mut stream: T,
    mut rx: Receiver<Data>,
    msg_bus: Sender<Message>,
) -> Result<()> {
    let r = async {
        loop {
            let mut buf = vec![0; 1024];
            match stream.read(&mut buf).await {
                Ok(0) => {
                    debug!("read buffer len = 0");
                    break;
                }
                Ok(n) => {
                    buf.truncate(n);
                    let data = Data {
                        id: 1,
                        conn_id,
                        data: buf,
                    };

                    if let Err(err) = msg_bus.send(Message::Data(data)).await {
                        error!("send message error: {}", err);

                        break;
                    }
                }
                Err(err) => {
                    error!("read error: {}", err);
                    return;
                }
            }

            // let data = Data {
            //     id: 1,
            //     conn_id: conn_id,
            //     data: buf,
            // };

            // if let Err(err) = msg_bus.send(Message::Data(data)).await {
            //     error!("send message error: {}", err);

            //     break;
            // }
        }
    };

    let w = async {
        while let Some(data) = rx.recv().await {
            info!("resciver data: {}", data.id);
            // stream.write_all(&data.data).await;
        }
    };

    let _ = tokio::join!(r, w);

    Ok(())
}
