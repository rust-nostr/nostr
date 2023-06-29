// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use std::time::Duration;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let client = Client::new(&my_keys);
    // Some relays may experience high load at times, so they may be unresponsive
    // Enable or disable the relays you'd like to use:
    // client.add_relay("wss://relay.nostr.info", None).await?;
    // client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nos.lol", None).await?;

    client.connect().await;

    // Publish a text note
    client.publish_text_note("Hello world", &[]).await?;

    // Create a text note POW event
    let event: Event = EventBuilder::new_text_note("POW text note from nostr-sdk", &[])
        .to_pow_event(&my_keys, 20)?;
    client.send_event(event).await?;

    let evs = client
        .get_all_events_of(
            vec![Filter::new().kind(Kind::TextNote).since(Timestamp::now())],
            Duration::from_secs(5),
        )
        .await?;
    println!("Live notes received within the past 5 seconds: {evs:#?}");

    Ok(())
}
