// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let database = NostrLMDB::open("./db/nostr-lmdb")?;
    let client: Client = Client::builder().database(database).build();
    client.add_relay("wss://relay.damus.io").await?;

    client.connect().await;

    let public_key =
        PublicKey::from_bech32("npub1080l37pfvdpyuzasyuy2ytjykjvq3ylr5jlqlg7tvzjrh9r8vn3sf5yaph")?;

    // ################ Aggregated query with same filter ################
    let filter = Filter::new()
        .author(public_key)
        .kind(Kind::TextNote)
        .limit(50);
    let stored_events = client.database().query(filter.clone()).await?;
    let fetched_events = client.fetch_events(filter, Duration::from_secs(10)).await?;
    let events = stored_events.merge(fetched_events);

    for event in events.into_iter() {
        println!("{}", event.as_json());
    }

    // ################ Aggregated query with different filters ################

    // Query events from database
    let filter = Filter::new().author(public_key).kind(Kind::TextNote);
    let stored_events = client.database().query(filter).await?;

    // Query events from relays
    let filter = Filter::new().author(public_key).kind(Kind::Metadata);
    let fetched_events = client.fetch_events(filter, Duration::from_secs(10)).await?;

    // Add temp relay and fetch other events
    client.add_relay("wss://nostr.oxtr.dev").await?;
    client.connect_relay("wss://nostr.oxtr.dev").await?;
    let filter = Filter::new().kind(Kind::ContactList).limit(100);
    let fetched_events_from = client
        .fetch_events_from(["wss://nostr.oxtr.dev"], filter, Duration::from_secs(10))
        .await?;
    client.force_remove_relay("wss://nostr.oxtr.dev").await?;

    // Aggregate results (can be done many times)
    let events = stored_events
        .merge(fetched_events)
        .merge(fetched_events_from);

    for event in events.into_iter() {
        println!("{}", event.as_json());
    }

    Ok(())
}
