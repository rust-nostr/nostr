// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_redb::NostrRedb;
use nostr_relay_builder::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let database = NostrRedb::in_memory()?;
    let tor = RelayBuilderHiddenService::new("rust-nostr-local-hs-test");
    let builder = RelayBuilder::new(database).tor(tor);

    let relay = LocalRelay::run(builder).await?;

    println!("Url: {}", relay.url());
    println!("Hidden service: {:?}", relay.hidden_service());

    // Keep up the program
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
