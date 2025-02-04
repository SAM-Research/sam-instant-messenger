pub mod error;
pub mod storage;

pub use error::ClientError;
pub use error::Result;

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
