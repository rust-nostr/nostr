// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr::secp256k1::SecretKey;
use nostr::util::nips::nip04::decrypt;
use nostr::util::nips::nip19::FromBech32;
use nostr::util::time::timestamp;
use nostr::{Keys, Kind, KindBase, SubscriptionFilter};
use nostr_sdk::{Client, RelayPoolNotifications, Result};

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    let client = Client::new_with_store(&my_keys, "./")?;

    client.restore_relays().await?;

    client.add_relay("ws://127.0.0.1:8080", None).await?;
    client.add_relay("wss://relay.nostr.info", proxy).await?;
    client.add_relay("wss://rsslay.fiatjaf.com", None).await?;
    client
        .add_relay(
            "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
            proxy,
        )
        .await?;

    client.connect().await;

    let subscription = SubscriptionFilter::new()
        .pubkey(my_keys.public_key())
        .since(timestamp());

    client.subscribe(vec![subscription]).await?;

    loop {
        let mut notifications = client.notifications();
        while let Ok(notification) = notifications.recv().await {
            if let RelayPoolNotifications::ReceivedEvent(event) = notification {
                if event.kind == Kind::Base(KindBase::EncryptedDirectMessage) {
                    if let Ok(msg) = decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content)
                    {
                        println!("New DM: {}", msg);
                    } else {
                        log::error!("Impossible to decrypt direct message");
                    }
                } else {
                    println!("{:?}", event);
                }
            }
        }
    }
}
