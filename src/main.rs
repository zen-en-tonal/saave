use std::{net::SocketAddr, str::FromStr};

use kvs::compression::CompressionKvs;
mod compression;
mod http;
mod kvs;

#[tokio::main]
async fn main() {
    let tree = sled::open("test.db").unwrap().open_tree("test").unwrap();
    let comp = CompressionKvs::new(tree);
    http::AppBuilder::new(comp)
        .run(SocketAddr::from_str("0.0.0.0:3000").unwrap())
        .await;
}
