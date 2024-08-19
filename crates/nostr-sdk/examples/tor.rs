// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Parse keys
    let my_keys = Keys::parse(BECH32_SK)?;

    // Configure client to use embedded tor for `.onion` relays
    let connection: Connection = Connection::new()
        .embedded_tor()
        .target(ConnectionTarget::Onion);
    let opts = Options::new().connection(connection);
    let client = Client::with_opts(&my_keys, opts);

    // Add relays
    client
        .add_relays([
            "wss://relay.damus.io",
            "ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion",
            "ws://2jsnlhfnelig5acq6iacydmzdbdmg7xwunm4xl6qwbvzacw4lwrjmlyd.onion",
        ])
        .await?;

    client.connect().await;

    let filter: Filter = Filter::new().pubkey(my_keys.public_key()).limit(0);
    client.subscribe(vec![filter], None).await?;

    // Handle subscription notifications with `handle_notifications` method
    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind == Kind::GiftWrap {
                    let UnwrappedGift { rumor, .. } = client.unwrap_gift_wrap(&event).await?;
                    println!("Rumor: {}", rumor.as_json());
                } else {
                    println!("{:?}", event);
                }
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}
