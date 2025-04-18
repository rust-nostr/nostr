// Copyright (c) 2025 Protom
// Distributed under the MIT software license

use std::time::Duration;

use nostr_database::prelude::*;
use nostr_postgresdb::NostrPostgres;
use nostr_relay_builder::prelude::*;

// Your database URL
const DB_URL: &str = "postgres://postgres:password@localhost:5432";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // This will programatically run pending db migrations
    nostr_postgresdb::run_migrations(DB_URL)?;

    // Create a conncetion pool
    let pool = nostr_postgresdb::postgres_connection_pool(DB_URL).await?;

    // Create a nostr db instance
    let db: NostrPostgres = pool.into();

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
