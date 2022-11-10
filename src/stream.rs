#[derive(Debug)]
pub struct MyStream {
    conn_id: i64,
    proto: String,
    addr: String,
}

impl MyStream {
    pub fn new(conn_id: i64, proto: &str, addr: &str) -> Self {
        MyStream {
            conn_id,
            proto: proto.to_string(),
            addr: addr.to_string(),
        }
    }
}
