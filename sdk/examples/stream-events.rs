// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let public_key =
        PublicKey::parse("npub1zfss807aer0j26mwp2la0ume0jqde3823rmu97ra6sgyyg956e0s6xw445")?;

    let client = Client::default();
    client.add_relay("wss://nos.lol").await?;
    client.add_relay("wss://user.kindpag.es").await?;
    client.add_relay("wss://purplepag.es").await?;
    client.add_relay("wss://relay.primal.net").await?;
    client.add_relay("wss://relay.damus.io").await?;

    client.connect().await;

    // Stream events from all connected relays
    let filter = Filter::new()
        .author(public_key)
        .kind(Kind::RelayList)
        .limit(1);

    let mut stream = client
        .stream_events(filter)
        .timeout(Duration::from_secs(15))
        .policy(ReqExitPolicy::ExitOnEOSE)
        .await?;

    while let Some((url, res)) = stream.next().await {
        let event = res?;
        println!("Received event from '{url}': {}", event.id);
    }

    Ok(())
}
