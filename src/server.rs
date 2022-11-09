use std::collections::HashMap;
use crate::{message::Message, session::Session};
use crate::{utils, Connect};
use anyhow::{anyhow, Ok, Result};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread::park;
use tokio::net::{TcpListener, TcpStream};
use tracing::info;

pub trait Authorizer {
    fn auth(&self, token: &str) -> bool;
}

pub struct Server<A: Authorizer> {
    ath: Arc<A>,
    addr: SocketAddr,
    sess: Arc<Mutex<HashMap<String, Arc<Session>>>>
}

impl<A> Server<A>
where
    A: Authorizer,
{
    pub fn new(ath: A, addr: SocketAddr) -> Self {
        Self {
            ath: Arc::new(ath),
            addr,
            sess: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn serve(&self) -> Result<()> {
        let ln = TcpListener::bind(&self.addr).await?;

        loop {
            let (stream, addr) = ln.accept().await?;
            // tokio::spawn(async move {
            let ret = self.process(stream).await;
            match ret {
                Err(err) => {
                    println!("ERR: {}, addr:{}", err, addr)
                }
                _ => {}
            }
            // });
        }
    }

    async fn process(&self, mut stream: TcpStream) -> Result<()> {
        let msg: Message = Message::read(&mut stream).await?;

        let token = match msg {
            Message::Token(token) => token,
            _ => return Err(anyhow!("invalid client token protocol")),
        };

        println!("new client connect, token: {}", token);

        let mut session = Arc::new(Session::new(token.clone(), rand::random(), stream, false));
        // session.serve().await?;

        let mut lock = self.sess.lock().unwrap();
        lock.insert(token, session);


        Ok(())
    }
}
