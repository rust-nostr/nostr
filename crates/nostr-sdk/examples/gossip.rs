// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let my_keys = Keys::parse(BECH32_SK)?;

    let opts = Options::new().gossip(true);
    let client = Client::with_opts(my_keys, opts);

    client.add_discovery_relay("wss://relay.damus.io").await?;
    client.add_discovery_relay("wss://purplepag.es").await?;
    //client.add_discovery_relay("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion").await?;

    // client.add_relay("wss://relay.snort.social").await?;
    // client.add_relay("wss://relay.damus.io").await?;

    client.connect().await;

    // Publish a text note
    let pubkey =
        PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;

    let output = client
        .publish_text_note(
            "Hello world nostr:npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
            [Tag::public_key(pubkey)],
        )
        .await?;
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
        .fetch_events(vec![filter], Some(Duration::from_secs(10)))
        .await?;

    for event in events.into_iter() {
        println!("{}", event.as_json());
    }

    Ok(())
}
