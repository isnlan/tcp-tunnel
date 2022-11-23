mod client;
mod message;
mod server;
mod session;
mod stream;
pub mod utils;

mod mutex;
pub use client::connect;
pub use message::*;
pub use server::*;
pub use stream::*;
pub use utils::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
