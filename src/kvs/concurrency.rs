use bytes::Bytes;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

use super::{Kvs, KvsError, Result};

pub struct ConcurrencyKvs {
    tx: Sender<Command<String, Bytes>>,
}

pub struct Runner<T: Kvs + Send + Sync> {
    inner: T,
    rx: Receiver<Command<String, Bytes>>,
}

impl<T: Kvs + Send + Sync> Runner<T> {
    pub async fn spawn(&mut self) {
        loop {
            while let Some(command) = self.rx.recv().await {
                match command {
                    Command::Get((key, rx)) => {
                        let res = self.inner.get(&key);
                        let _ = rx.send(res);
                    }
                    Command::Insert((key, value, rx)) => {
                        let res = self.inner.insert(key, value);
                        let _ = rx.send(res);
                    }
                    Command::Remove((key, rx)) => {
                        let res = self.inner.remove(&key);
                        let _ = rx.send(res);
                    }
                }
            }
        }
    }
}

impl ConcurrencyKvs {
    pub fn new<T: Kvs + Send + Sync>(kvs: T, num_buffer: usize) -> (Self, Runner<T>) {
        let (tx, rx) = mpsc::channel(num_buffer);
        (Self { tx }, Runner { inner: kvs, rx })
    }

    pub fn make_handle(&self) -> Handle<String, Bytes> {
        Handle {
            tx: self.tx.clone(),
        }
    }
}

pub struct Handle<K, V> {
    tx: Sender<Command<K, V>>,
}

enum Command<K, V> {
    Get((K, oneshot::Sender<Result<Option<V>>>)),
    Insert((K, V, oneshot::Sender<Result<Option<V>>>)),
    Remove((K, oneshot::Sender<Result<Option<V>>>)),
}

impl<K: 'static, V: 'static> Handle<K, V> {
    pub async fn get(&self, key: K) -> Result<Option<V>> {
        let (tx, rx) = oneshot::channel::<Result<Option<V>>>();
        let command = Command::Get((key, tx));
        self.tx
            .send(command)
            .await
            .map_err(|e| KvsError::Other(e.to_string()))?;
        rx.await.map_err(|e| KvsError::Other(e.to_string()))?
    }

    pub async fn insert(&self, key: K, value: V) -> Result<Option<V>> {
        let (tx, rx) = oneshot::channel::<Result<Option<V>>>();
        let command = Command::Insert((key, value, tx));
        self.tx
            .send(command)
            .await
            .map_err(|e| KvsError::Other(e.to_string()))?;
        rx.await.map_err(|e| KvsError::Other(e.to_string()))?
    }

    pub async fn remove(&self, key: K) -> Result<Option<V>> {
        let (tx, rx) = oneshot::channel::<Result<Option<V>>>();
        let command = Command::Remove((key, tx));
        self.tx
            .send(command)
            .await
            .map_err(|e| KvsError::Other(e.to_string()))?;
        rx.await.map_err(|e| KvsError::Other(e.to_string()))?
    }
}
