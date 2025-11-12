// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_database::prelude::*;
use nostr_relay_builder::prelude::*;
use nostr_sqldb::{NostrSql, NostrSqlBackend};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let backend = NostrSqlBackend::sqlite("nostr.db");

    // Create a nostr db instance and run pending db migrations if any
    let db = NostrSql::new(backend).await?;

    // Add db to builder
    let builder = RelayBuilder::default().database(db);

    // Create local relay
    let relay = LocalRelay::run(builder).await?;
    println!("Url: {}", relay.url());

    // Keep up the program
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
