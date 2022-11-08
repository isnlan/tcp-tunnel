mod client;
mod message;
mod protocol;
mod server;
mod session;
pub mod utils;

pub use client::connect;
pub use message::*;
pub use server::*;
pub use utils::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
