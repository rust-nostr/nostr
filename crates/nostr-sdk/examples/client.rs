// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let client = Client::new(keys);

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://relay.rip").await?;

    client.connect().await;

    // Publish a text note
    let builder = EventBuilder::text_note("Hello world", []);
    let output = client.send_event_builder(builder).await?;
    println!("Event ID: {}", output.id().to_bech32()?);
    println!("Sent to: {:?}", output.success);
    println!("Not sent to: {:?}", output.failed);

    // Create a text note POW event to relays
    let builder = EventBuilder::text_note("POW text note from rust-nostr", []).pow(20);
    client.send_event_builder(builder).await?;

    // Send a text note POW event to specific relays
    let builder = EventBuilder::text_note("POW text note from rust-nostr 16", []).pow(16);
    client
        .send_event_builder_to(["wss://relay.damus.io", "wss://relay.rip"], builder)
        .await?;

    Ok(())
}
