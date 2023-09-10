pub mod compression;
pub mod concurrency;
pub mod sled;

use std::{error::Error, fmt::Display, result};

pub use bytes::Bytes;

pub type Result<T> = result::Result<T, KvsError>;

pub trait Kvs {
    fn get(&self, key: &impl AsRef<str>) -> Result<Option<Bytes>>;
    fn insert(&mut self, key: impl AsRef<str>, value: Bytes) -> Result<Option<Bytes>>;
    fn remove(&mut self, key: &impl AsRef<str>) -> Result<Option<Bytes>>;
}

#[derive(Debug)]
pub enum KvsError {
    Io(std::io::Error),
    Other(String),
}

impl Display for KvsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letter = match self {
            KvsError::Io(e) => format!("io error; {}", e),
            KvsError::Other(e) => format!("other error; {}", e),
        };
        write!(f, "{letter}")
    }
}

impl Error for KvsError {}

impl From<std::io::Error> for KvsError {
    fn from(value: std::io::Error) -> Self {
        KvsError::Io(value)
    }
}
