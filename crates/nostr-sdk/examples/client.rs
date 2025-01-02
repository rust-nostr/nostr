// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let client = Client::new(keys);

    client.add_relay("udp://239.19.88.1:9797").await?;

    client.connect().await;

    loop {
        let builder = EventBuilder::text_note("Hello world");
        let output = client.send_event_builder(builder).await?;
        println!("Event ID: {}", output.id().to_bech32()?);
        println!("Sent to: {:?}", output.success);
        println!("Not sent to: {:?}", output.failed);

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
