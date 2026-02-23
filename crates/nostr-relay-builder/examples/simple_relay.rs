use std::time::Duration;

use nostr_lmdb::NostrLmdb;
use nostr_relay_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db = NostrLmdb::open("./db/nostr-lmdb").await?;

    let relay = LocalRelay::builder().port(7777).database(db).build()?;

    relay.run().await?;

    let url = relay.url().await;
    println!("Url: {url}");

    // Keep up the program
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
