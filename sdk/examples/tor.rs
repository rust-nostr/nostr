// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Configure client to use a proxy for the onion relays
    let proxy: Proxy = Proxy::onion(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));
    let client = Client::builder().proxy(proxy).build();

    // Add relays
    client.add_relay("wss://relay.damus.io").await?;
    client
        .add_relay("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion")
        .await?;

    client.connect().await;

    // Parse keys
    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;

    let filter: Filter = Filter::new().pubkey(keys.public_key()).limit(0);
    client.subscribe(filter).await?;

    let mut notifications = client.notifications();

    while let Some(notification) = notifications.next().await {
        if let ClientNotification::Event { event, .. } = notification {
            if event.kind == Kind::GiftWrap {
                let UnwrappedGift { rumor, .. } = UnwrappedGift::from_gift_wrap(&keys, &event)?;
                println!("Rumor: {}", rumor.as_json());
            } else {
                println!("{:?}", event);
            }
        }
    }

    Ok(())
}
