pub mod compression;
pub mod concurrency;
pub mod sled;

use std::result;

pub use bytes::Bytes;

pub type Result<T> = result::Result<T, KvsError>;

pub trait Kvs {
    fn get(&self, key: &impl AsRef<str>) -> Result<Option<Bytes>>;
    fn insert(&mut self, key: impl AsRef<str>, value: Bytes) -> Result<Option<Bytes>>;
    fn remove(&mut self, key: &impl AsRef<str>) -> Result<Option<Bytes>>;
}

pub struct KvsError;
