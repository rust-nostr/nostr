// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let database = RocksDatabase::open("./db/rocksdb").await?;
    let client: Client = ClientBuilder::new(&my_keys).database(database).build();

    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nostr.wine", None).await?;
    client.add_relay("wss://atl.purplerelay.com", None).await?;

    client.connect().await;

    /* // Publish a text note
    client.publish_text_note("Hello world", &[]).await?; */

    // Negentropy reconcile
    let filter = Filter::new().author(my_keys.public_key());
    client
        .reconcile(filter, NegentropyOptions::default())
        .await?;

    // Query events from database
    let filter = Filter::new().author(my_keys.public_key()).limit(10);
    let events = client.database().query(vec![filter]).await?;
    println!("Events: {events:?}");

    Ok(())
}
