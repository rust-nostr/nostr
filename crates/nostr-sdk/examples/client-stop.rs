// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use async_utility::thread;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    client.add_relay("wss://nostr.oxtr.dev").await?;
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.openchain.fr").await?;

    client.connect().await;

    thread::sleep(Duration::from_secs(10)).await;

    client.stop().await?;

    thread::sleep(Duration::from_secs(15)).await;

    client.start().await;

    thread::sleep(Duration::from_secs(10)).await;

    client.publish_text_note("Test", []).await?;

    Ok(())
}
