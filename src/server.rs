use crate::stream;
use crate::{message::Message, session::Session};

use anyhow::{anyhow, Ok, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::task;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::info;

pub trait Authorizer {
    fn auth(&self, token: &str) -> bool;
}

pub struct Server<A: Authorizer> {
    ath: Arc<A>,
    addr: SocketAddr,
    sess: Arc<Mutex<HashMap<String, Arc<Session>>>>,
}

impl<A> Server<A>
where
    A: Authorizer,
{
    pub fn new(ath: A, addr: SocketAddr) -> Self {
        Self {
            ath: Arc::new(ath),
            addr,
            sess: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn serve(&self) -> Result<()> {
        let ln = TcpListener::bind(&self.addr).await?;

        loop {
            let (stream, addr) = ln.accept().await?;

            if let Err(err) = self.process(stream).await {
                info!("ERR: {}, addr:{}", err, addr)
            }
        }
    }

    async fn process(&self, mut stream: TcpStream) -> Result<()> {
        let msg: Message = Message::read(&mut stream).await?;

        let token = match msg {
            Message::Token(token) => token,
            _ => return Err(anyhow!("invalid client token protocol")),
        };

        info!("new client connect, token: {}", token);

        let session = Arc::new(Session::new(token.clone(), rand::random(), stream, false));

        let sess_c = session.clone();
        task::spawn(async move {
            let _ = sess_c.serve().await;
        });

        let mut lock = self.sess.lock().await;
        lock.insert(token, session);

        Ok(())
    }

    pub async fn get_stream(
        &self,
        token: &str,
        proto: &str,
        addr: &str,
    ) -> Result<Option<stream::Stream>> {
        let lock = self.sess.lock().await;
        match lock.get(token) {
            Some(session) => {
                let stream = session.get_stream(proto, addr).await?;
                Ok(Some(stream))
            }
            None => Ok(Option::None),
        }
    }
}
