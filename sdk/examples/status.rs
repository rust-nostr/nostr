// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://relay.rip").await?;

    client.connect().await;

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;

    // Send a General statuses event to relays
    let general = LiveStatus::new(StatusType::General);
    let event = EventBuilder::live_status(general, "Building rust-nostr").sign(&keys)?;
    client.send_event(&event).await?;

    // Send a Music statuses event to relays
    let music = LiveStatus {
        status_type: StatusType::Music,
        expiration: Some(Timestamp::now() + Duration::from_secs(60 * 60 * 24)),
        reference: Some("spotify:search:Intergalatic%20-%20Beastie%20Boys".into()),
    };
    let event = EventBuilder::live_status(music, "Intergalatic - Beastie Boys").sign(&keys)?;
    client.send_event(&event).await?;

    Ok(())
}
