// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let my_keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;

    let database = NostrLMDB::open("./db/nostr-lmdb")?;
    let client: Client = ClientBuilder::default()
        .signer(&my_keys)
        .database(database)
        .build();

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://nostr.oxtr.dev").await?;

    client.connect().await;

    // Publish a text note
    client.publish_text_note("Hello world", []).await?;

    // Negentropy reconcile
    let filter = Filter::new().author(my_keys.public_key());
    let output = client
        .sync(
            filter,
            NegentropyOptions::default().direction(NegentropyDirection::Down),
        )
        .await?;

    println!("Local: {}", output.local.len());
    println!("Remote: {}", output.remote.len());
    println!("Sent: {}", output.sent.len());
    println!("Received: {}", output.received.len());
    println!("Failures:");
    for (url, map) in output.send_failures.iter() {
        println!("* '{url}':");
        for (id, e) in map.iter() {
            println!("  - {id}: {e}");
        }
    }

    // Query events from database
    let filter = Filter::new().author(my_keys.public_key()).limit(10);
    let events = client.database().query(vec![filter]).await?;
    println!("Events: {events:?}");

    Ok(())
}
