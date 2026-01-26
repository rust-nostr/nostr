// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_gossip_memory::prelude::*;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let gossip = NostrGossipMemory::unbounded();
    let client = Client::builder()
        .signer(keys.clone())
        .gossip(gossip)
        .build();

    // Add discovery relays
    client
        .add_relay("wss://relay.damus.io")
        .capabilities(RelayCapabilities::DISCOVERY)
        .await?;
    client
        .add_relay("wss://purplepag.es")
        .capabilities(RelayCapabilities::DISCOVERY)
        .await?;

    client.connect().await;

    // Publish a text note
    let pubkey =
        PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;

    let event = EventBuilder::text_note(
        "Hello world nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
    )
    .tag(Tag::public_key(pubkey))
    .sign_with_keys(&keys)?;
    let output = client.send_event(&event).await?;
    println!("Event ID: {}", output.to_bech32()?);

    println!("Sent to:");
    for url in output.success.into_iter() {
        println!("- {url}");
    }

    println!("Not sent to:");
    for (url, reason) in output.failed.into_iter() {
        println!("- {url}: {reason:?}");
    }

    // Get events
    let filter = Filter::new().author(pubkey).kind(Kind::TextNote).limit(3);
    let events = client
        .fetch_events(filter)
        .timeout(Duration::from_secs(10))
        .await?;

    for event in events.into_iter() {
        println!("{}", event.as_json());
    }

    Ok(())
}
