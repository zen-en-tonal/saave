use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::compression::{Compression, Compressor};

use super::Kvs;

pub struct CompressionKvs<T> {
    inner: T,
}

impl<T> CompressionKvs<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Kvs> Kvs for CompressionKvs<T> {
    fn get(&self, key: &impl AsRef<str>) -> super::Result<Option<Bytes>> {
        match self.inner.get(key)? {
            Some(value) => {
                let mime = mime_db::lookup(&key).unwrap_or("application/octet-stream");
                let comp = Compressor::new(mime);
                let mut buf = BytesMut::new().writer();
                comp.decompress(value.clone().reader(), &mut buf).unwrap();
                Ok(Some(buf.into_inner().into()))
            }
            None => Ok(None),
        }
    }

    fn insert(&mut self, key: impl AsRef<str>, value: Bytes) -> super::Result<Option<Bytes>> {
        let mime = mime_db::lookup(&key).unwrap_or("application/octet-stream");
        let comp = Compressor::new(mime);
        let mut buf = BytesMut::new().writer();
        comp.compress(value.reader(), &mut buf).unwrap();
        self.inner.insert(key, buf.into_inner().into())
    }

    fn remove(&mut self, key: &impl AsRef<str>) -> super::Result<Option<Bytes>> {
        self.inner.remove(key)
    }
}
