// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let client = Client::new(keys);

    client.add_relay("ws://127.0.0.1:17445").await?;

    client.connect().await;

    // Publish a text note
    let builder = EventBuilder::text_note("Hello world");
    let output = client.send_event_builder(builder).await?;
    println!("Event ID: {}", output.id().to_bech32()?);
    println!("Sent to: {:?}", output.success);
    println!("Not sent to: {:?}", output.failed);

    let events = client
        .fetch_events(Filter::new().kind(Kind::TextNote), Duration::from_secs(10))
        .await?;

    for event in events {
        println!("{}", event.as_json())
    }

    Ok(())
}
