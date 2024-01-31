// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let client = Client::new(&my_keys);
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://relay.rip").await?;

    client.connect().await;

    // Publish a text note
    client.publish_text_note("Hello world", []).await?;

    // Create a text note POW event and broadcast to all connected relays
    let event: Event =
        EventBuilder::text_note("POW text note from nostr-sdk", []).to_pow_event(&my_keys, 20)?;
    client.send_event(event).await?;

    // Send multiple events at once (to all relays)
    let mut events: Vec<Event> = Vec::new();
    for i in 0..10 {
        events.push(EventBuilder::text_note(format!("Event #{i}"), []).to_event(&my_keys)?);
    }
    let opts = RelaySendOptions::default();
    client.batch_event(events, opts).await?;

    // Send event to specific relays
    let event: Event = EventBuilder::text_note("POW text note from nostr-sdk 16", [])
        .to_pow_event(&my_keys, 16)?;
    client
        .send_event_to(["wss://relay.damus.io", "wss://relay.rip"], event)
        .await?;

    Ok(())
}
