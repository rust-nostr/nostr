// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;

    let database = NostrLMDB::open("./db/nostr-lmdb")?;
    let client: Client = ClientBuilder::default()
        .signer(keys.clone())
        .database(database)
        .build();

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://nostr.oxtr.dev").await?;

    client.connect().await;

    // Publish a text note
    let builder = EventBuilder::text_note("Hello world");
    client.send_event_builder(builder).await?;

    // Negentropy sync
    let filter = Filter::new().author(keys.public_key());
    let (tx, mut rx) = SyncProgress::channel();
    let opts = SyncOptions::default().progress(tx);

    tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            let progress = *rx.borrow_and_update();
            if progress.total > 0 {
                println!("{:.2}%", progress.percentage() * 100.0);
            }
        }
    });
    let output = client.sync(filter, &opts).await?;

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
    let filter = Filter::new().author(keys.public_key()).limit(10);
    let events = client.database().query(filter).await?;
    println!("Events: {events:?}");

    // Database downcasting to access to specific APIs
    if let Some(_lmdb) = client.database().downcast_ref::<NostrLMDB>() {
        // Access specific APIs here
    }

    Ok(())
}
