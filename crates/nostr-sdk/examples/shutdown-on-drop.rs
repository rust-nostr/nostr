// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use async_utility::thread;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let opts = Options::new().shutdown_on_drop(true);
    let client = ClientBuilder::new().opts(opts).build();
    client.add_relay("wss://relay.nostr.info").await?;
    client.add_relay("wss://relay.damus.io").await?;

    client.connect().await;

    let c = client.clone();
    thread::spawn(async move {
        thread::sleep(Duration::from_secs(3)).await;
        c.relays().await;
        // First drop, dropping client...
    });

    thread::sleep(Duration::from_secs(10)).await;

    // Try to publish a text note (will fail since the client is dropped)
    client.publish_text_note("Hello world", []).await?;

    Ok(())
}
