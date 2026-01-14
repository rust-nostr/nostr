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

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://relay.rip").await?;

    client.connect().await;

    // Send a General statuses event to relays
    let general = LiveStatus::new(StatusType::General);
    let builder = EventBuilder::live_status(general, "Building rust-nostr");
    client.send_event_builder(builder).await?;

    // Send a Music statuses event to relays
    let music = LiveStatus {
        status_type: StatusType::Music,
        expiration: Some(Timestamp::now() + Duration::from_secs(60 * 60 * 24)),
        reference: Some("spotify:search:Intergalatic%20-%20Beastie%20Boys".into()),
    };
    let builder = EventBuilder::live_status(music, "Intergalatic - Beastie Boys");
    client.send_event_builder(builder).await?;

    Ok(())
}
