mod client;
mod protocol;
mod server;
pub mod pack;

pub use server::*;
pub use pack::*;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
