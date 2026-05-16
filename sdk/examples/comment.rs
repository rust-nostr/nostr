// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    client.add_relay("wss://relay.damus.io/").await?;
    client.add_relay("wss://relay.primal.net/").await?;

    client.connect().await;

    let event_id =
        EventId::from_bech32("note1hrrgx2309my3wgeecx2tt6fl2nl8hcwl0myr3xvkcqpnq24pxg2q06armr")?;
    let events = client
        .fetch_events(Filter::new().id(event_id))
        .timeout(Duration::from_secs(10))
        .await?;

    let keys = Keys::generate();

    let comment_to = events.first().unwrap();
    let event = EventBuilder::comment("This is a reply", CommentTarget::from(comment_to), None)
        .finalize(&keys)?;

    let output = client.send_event(&event).await?;
    println!("Output: {:?}", output);

    Ok(())
}
