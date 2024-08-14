// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let public_key =
        PublicKey::from_bech32("npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c")?;

    let client = Client::default();
    client.add_relay("wss://purplepag.es").await?;
    client.connect().await;

    let filter = Filter::new().author(public_key).kind(Kind::RelayList);
    let events: Vec<Event> = client
        .get_events_of(
            vec![filter],
            EventSource::relays(Some(Duration::from_secs(10))),
        )
        .await?;
    let event = events.first().unwrap();
    println!("Found relay list metadata:");
    for (url, metadata) in nip65::extract_relay_list(event) {
        println!("{url}: {metadata:?}");
    }

    Ok(())
}
