// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_relay_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let tor = RelayBuilderHiddenService::new("rust-nostr-local-hs-test");
    let builder = RelayBuilder::default().tor(tor);

    let relay = LocalRelay::new(builder)?;

    relay.run().await?;

    println!("Url: {}", relay.url().await);
    println!("Hidden service: {:?}", relay.hidden_service().await?);

    // Keep up the program
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
