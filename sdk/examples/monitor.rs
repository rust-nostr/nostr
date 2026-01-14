// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let monitor = Monitor::new(4096);
    let client = Client::builder().monitor(monitor).build();

    // Subscribe to monitor notifications
    let mut notifications = client.monitor().unwrap().subscribe();

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://relay.rip").await?;

    client.connect().await;

    while let Ok(notification) = notifications.recv().await {
        match notification {
            MonitorNotification::StatusChanged { relay_url, status } => {
                println!("Relay status changed for {relay_url}: {status}")
            }
        }
    }

    Ok(())
}
