// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::parse(BECH32_SK)?;

    let database = NdbDatabase::open("./db/ndb")?;
    let client: Client = Client::builder()
        .signer(keys.clone())
        .database(database)
        .build();

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://atl.purplerelay.com").await?;
    client.connect().await;

    // Publish a text note
    client.publish_text_note("Hello world", []).await?;

    // Negentropy reconcile
    let filter = Filter::new().author(keys.public_key());
    client.sync(filter, &SyncOptions::default()).await?;

    // Query events from database
    let filter = Filter::new().author(keys.public_key()).limit(10);
    let events = client.database().query(vec![filter]).await?;
    println!("Events: {events:?}");

    Ok(())
}
