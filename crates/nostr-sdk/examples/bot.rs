// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    client.add_relay("udp://239.19.88.1:9797").await?;

    client.connect().await;

    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event { event, .. } = notification {
                println!("Received event: {}", event.as_json());
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}
