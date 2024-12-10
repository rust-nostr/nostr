// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::generate();
    let client = Client::builder().signer(keys).build();

    client.add_relay("wss://relay.damus.io/").await?;
    client.add_relay("wss://relay.primal.net/").await?;

    client.connect().await;

    let event_id =
        EventId::from_bech32("note1hrrgx2309my3wgeecx2tt6fl2nl8hcwl0myr3xvkcqpnq24pxg2q06armr")?;
    let events = client
        .fetch_events(vec![Filter::new().id(event_id)], Duration::from_secs(10))
        .await?;

    let comment_to = events.first().unwrap();
    let builder = EventBuilder::comment("This is a reply", comment_to, None, None);

    let output = client.send_event_builder(builder).await?;
    println!("Output: {:?}", output);

    Ok(())
}
