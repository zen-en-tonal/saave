use bytes::Bytes;

use super::Kvs;

impl Kvs for sled::Tree {
    fn get(&self, key: &impl AsRef<str>) -> super::Result<Option<bytes::Bytes>> {
        match self.get(key.as_ref()) {
            Ok(option) => Ok(option.map(|some| Bytes::copy_from_slice(some.as_ref()))),
            Err(_) => Err(super::KvsError),
        }
    }

    fn insert(
        &mut self,
        key: impl AsRef<str>,
        value: bytes::Bytes,
    ) -> super::Result<Option<bytes::Bytes>> {
        let res = match sled::Tree::insert(&self, key.as_ref(), value.to_vec()) {
            Ok(option) => Ok(option.map(|some| Bytes::copy_from_slice(some.as_ref()))),
            Err(_) => Err(super::KvsError),
        };
        res
    }

    fn remove(&mut self, key: &impl AsRef<str>) -> super::Result<Option<bytes::Bytes>> {
        let res = match sled::Tree::remove(&self, key.as_ref()) {
            Ok(option) => Ok(option.map(|some| Bytes::copy_from_slice(some.as_ref()))),
            Err(_) => Err(super::KvsError),
        };
        res
    }
}
