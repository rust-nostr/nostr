// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use clap::Parser;
use nostr_sdk::prelude::*;
use std::time::Duration;

#[derive(Parser)]
struct Args {
    #[structopt(
        name = "secret",
        long,
        default_value = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e"
    )]
    /// Nostr secret key
    secret: String,
    #[structopt(name = "username", long, default_value = "nostr-rs user")]
    /// Nostr username
    username: String,
    #[structopt(name = "displayname", long, default_value = "nostr-rs user")]
    /// Nostr display name
    displayname: String,
    #[structopt(name = "about", long, default_value = "nostr-rs user")]
    /// Nostr about string
    about: Option<String>,
    #[structopt(
        name = "picture",
        long,
        default_value = "https://robohash.org/nostr-rs"
    )]
    /// picture url
    picture: Option<String>,
    #[structopt(name = "banner", long, default_value = "https://robohash.org/nostr-rs")]
    /// banner url
    banner: Option<String>,
    #[structopt(name = "nip05", long, default_value = "username@example.com")]
    /// nip05
    nip05: Option<String>,
    #[structopt(name = "lud16", long, default_value = "pay@yukikishimoto.com")]
    /// lud16
    lud16: Option<String>,
}

#[tokio::main]
async fn run() -> Result<()> {
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
    let stored_events = client.database().query(vec![filter.clone()]).await?;
    let fetched_events = client
        .fetch_events(vec![filter], Duration::from_secs(10))
        .await?;
    let events = stored_events.merge(fetched_events);

    for event in events.into_iter() {
        println!("{}", event.as_json());
    }

    // ################ Aggregated query with different filters ################

    // Query events from database
    let filter = Filter::new().author(public_key).kind(Kind::TextNote);
    let stored_events = client.database().query(vec![filter]).await?;

    // Query events from relays
    let filter = Filter::new().author(public_key).kind(Kind::Metadata);
    let fetched_events = client
        .fetch_events(vec![filter], Duration::from_secs(10))
        .await?;

    // Add temp relay and fetch other events
    client.add_relay("wss://nostr.oxtr.dev").await?;
    client.connect_relay("wss://nostr.oxtr.dev").await?;
    let filter = Filter::new().kind(Kind::ContactList).limit(100);
    let fetched_events_from = client
        .fetch_events_from(
            ["wss://nostr.oxtr.dev"],
            vec![filter],
            Some(Duration::from_secs(10)).unwrap(),
        )
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
fn main() {
    let _ = run();
}
