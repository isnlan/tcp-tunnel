use std::net::SocketAddr;
use std::thread::park;
use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use crate::pack;

pub trait Authorizer {
    fn auth(&self, token :&str) -> bool;
}

pub struct Server<A: Authorizer>{
    ath: A,
    addr: SocketAddr
}

impl <A>Server<A> where A : Authorizer {
    pub fn new(ath: A, addr: SocketAddr) -> Self {
        Self{
            ath,
            addr
        }
    }

    pub async fn serve(&self) -> Result<()> {
        let ln = TcpListener::bind(&self.addr).await?;

        loop {
            let (stream, addr) = ln.accept().await?;
            tokio::spawn(async move {
                process(stream).await;
            });
        }
        Ok(())
    }
}

async fn process(mut stream: TcpStream) -> Result<()> {
    println!("----");

    loop {
        let data = pack::read_data(&mut stream).await?;
        let s = String::from_utf8_lossy(&data);

        println!("--> {:}", s);
    }
}


