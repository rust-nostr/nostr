// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_relay_builder::prelude::*;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Set the tracing level using an environment variable or a default.
    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    // Create a subscriber that formats and outputs tracing events.
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    let tor = RelayBuilderHiddenService::new("rust-nostr-local-hs-test");
    let builder = RelayBuilder::default().tor(tor);

    let relay = LocalRelay::run(builder).await?;

    tracing::info!("Url: {}", relay.url());
    tracing::info!("Hidden service: {:?}", relay.hidden_service());

    // Keep up the program
    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;
        tracing::debug!("Url: {}", relay.url());
        tracing::debug!("Hidden service: {:?}", relay.hidden_service());
    }
}
