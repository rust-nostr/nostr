use std::time::Duration;

use nostr_lmdb::NostrLMDB;
use nostr_relay_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db = NostrLMDB::open("./db/nostr-lmdb").await?;

    let builder = RelayBuilder::default().port(7777).database(db);

    let relay = LocalRelay::new(builder);

    relay.run().await?;

    let url = relay.url().await;
    println!("Url: {url}");

    // Keep up the program
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
