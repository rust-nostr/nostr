// Copyright (c) 2025 Protom
// Distributed under the MIT software license

use std::time::Duration;

use nostr_database::prelude::*;
use nostr_relay_builder::prelude::*;
use nostr_sqldb::NostrPostgres;

// Your database URL
const DB_URL: &str = "postgres://postgres:password@localhost:5432";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Create a nostr db instance and run pending db migrations if any
    let db = NostrPostgres::new(DB_URL).await?;

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
