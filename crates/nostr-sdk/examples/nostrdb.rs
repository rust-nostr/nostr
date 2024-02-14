// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let my_keys = Keys::parse(BECH32_SK)?;

    let database = NdbDatabase::open("./db/ndb")?;
    let client: Client = ClientBuilder::default()
        .signer(&my_keys)
        .database(database)
        .build();

    client
        .add_relays([
            "wss://relay.damus.io",
            "wss://nostr.wine",
            "wss://atl.purplerelay.com",
        ])
        .await?;
    client.connect().await;

    // Publish a text note
    client.publish_text_note("Hello world", []).await?;

    // Negentropy reconcile
    let filter = Filter::new().author(my_keys.public_key());
    client
        .reconcile(filter, NegentropyOptions::default())
        .await?;

    // Query events from database
    let filter = Filter::new().author(my_keys.public_key()).limit(10);
    let events = client.database().query(vec![filter], Order::Desc).await?;
    println!("Events: {events:?}");

    Ok(())
}
