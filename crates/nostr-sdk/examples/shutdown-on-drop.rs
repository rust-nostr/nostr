// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use async_utility::thread;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default(); // Counter set to 1

    client.add_relay("wss://relay.rip").await?;
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    let c = client.clone(); // Clone, counter set to 2
    let _ = thread::spawn(async move {
        thread::sleep(Duration::from_secs(3)).await;
        c.relays().await;
        // First drop, decrease counter to 1...
    });

    thread::sleep(Duration::from_secs(5)).await;

    let builder = EventBuilder::text_note("Hello world", []);
    client.send_event_builder(builder).await?;

    thread::sleep(Duration::from_secs(5)).await;

    Ok(())
}

// Client dropped, counter set to 0: auto shutdown relay pool.
