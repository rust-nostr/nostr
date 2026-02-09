// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Parse keys
    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;

    // Configure client to use embedded tor for `.onion` relays
    let connection: Connection = Connection::new()
        .proxy(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)))
        .target(ConnectionTarget::Onion);
    let client = Client::builder()
        .signer(keys.clone())
        .connection(connection)
        .build();

    // Add relays
    client.add_relay("wss://relay.damus.io").await?;
    client
        .add_relay("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion")
        .await?;
    client
        .add_relay("ws://2jsnlhfnelig5acq6iacydmzdbdmg7xwunm4xl6qwbvzacw4lwrjmlyd.onion")
        .await?;

    client.connect().await;

    let filter: Filter = Filter::new().pubkey(keys.public_key()).limit(0);
    client.subscribe(filter).await?;

    let mut notifications = client.notifications();

    while let Some(notification) = notifications.next().await {
        if let ClientNotification::Event { event, .. } = notification {
            if event.kind == Kind::GiftWrap {
                let UnwrappedGift { rumor, .. } =
                    UnwrappedGift::from_gift_wrap(&keys, &event).await?;
                println!("Rumor: {}", rumor.as_json());
            } else {
                println!("{:?}", event);
            }
        }
    }

    Ok(())
}
