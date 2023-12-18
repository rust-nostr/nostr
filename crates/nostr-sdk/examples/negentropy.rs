// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let client = Client::new(&my_keys);
    client.add_relay("wss://atl.purplerelay.com").await?;

    client.connect().await;

    let my_items = Vec::new();
    let filter = Filter::new().author(my_keys.public_key()).limit(10);
    let relay = client.relay("wss://atl.purplerelay.com").await?;
    let opts = NegentropyOptions::default();
    relay.reconcile(filter, my_items, opts).await?;

    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event { event, .. } = notification {
                println!("{:?}", event);
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}
