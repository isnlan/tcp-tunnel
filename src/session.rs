use crate::Message;
use anyhow::Result;
use std::collections::HashMap;
use tokio::net::TcpStream;

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

    pub fn serve() {
        loop {}
    }

    fn serveMessage(&self, msg: Message) -> Result<()> {
        Ok(())
    }
}

pub struct Connection {}
