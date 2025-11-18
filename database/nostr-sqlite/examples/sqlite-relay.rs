// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_relay_builder::prelude::*;
use nostr_sqlite::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Create a nostr db instance and run pending db migrations if any
    let db = NostrSqlite::open("nostr.sqlite").await?;

    // Create relay
    let builder = RelayBuilder::default().database(db);
    let relay = LocalRelay::new(builder);

    relay.run().await?;

    println!("Url: {}", relay.url().await);

    // Keep up the program
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
