// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let public_key =
        PublicKey::from_bech32("npub1080l37pfvdpyuzasyuy2ytjykjvq3ylr5jlqlg7tvzjrh9r8vn3sf5yaph")?;

    let database = RocksDatabase::open("./db/rocksdb").await?;
    let client: Client = Client::builder().database(database).build();

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://atl.purplerelay.com").await?;

    client.connect().await;

    // Negentropy reconcile
    let filter = Filter::new().author(public_key);
    client
        .reconcile(filter, NegentropyOptions::default())
        .await?;

    // Query events from database
    let filter = Filter::new().author(public_key).limit(10);
    let events = client.database().query(vec![filter], Order::Desc).await?;
    println!("Events: {events:?}");

    Ok(())
}
