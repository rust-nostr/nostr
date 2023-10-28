// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let client = Client::new(&my_keys);
    client.add_relay("wss://relay.damus.io", None).await?;

    client.connect().await;

    let my_items = Vec::new();
    let filter = Filter::new().author(my_keys.public_key()).limit(10);
    let relay = client.relay("wss://relay.damus.io").await?;
    relay
        .reconcilie(filter, my_items, Duration::from_secs(30))
        .await?;

    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event(_url, event) = notification {
                println!("{:?}", event);
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}
