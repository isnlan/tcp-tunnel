mod client;
mod message;
mod server;
mod session;
mod stream;
mod utils;

pub use client::connect;
pub use server::{Authorizer, Server};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
