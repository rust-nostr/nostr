// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate nostr_sdk;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use anyhow::Result;
use chrono::Utc;
use nostr_sdk::base::util::nip04::decrypt;
use nostr_sdk::base::{Keys, Kind, KindBase, SubscriptionFilter};
use nostr_sdk::{Client, RelayPoolNotifications};

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let my_keys = Keys::new_from_bech32(BECH32_SK)?;

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    let mut client = Client::new(&my_keys, None);
    client.add_relay("wss://relay.nostr.info", None).await?;
    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nostr.openchain.fr", None).await?;
    client
        .add_relay(
            "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
            proxy,
        )
        .await?;

    client.connect_all().await?;

    client
        .delete_event("57689882a98ac4db67933196c121489dea7e1231f7c0f20accad4de838500edc")
        .await?;

    let subscription = SubscriptionFilter::new()
        .pubkey(my_keys.public_key)
        .since(Utc::now());

    client.subscribe(vec![subscription]).await?;

    client
        .handle_notifications(|notification| {
            match notification {
                RelayPoolNotifications::ReceivedEvent(event) => {
                    if event.kind == Kind::Base(KindBase::EncryptedDirectMessage) {
                        if let Ok(msg) =
                            decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content)
                        {
                            println!("New DM: {}", msg);
                        } else {
                            log::error!("Impossible to decrypt direct message");
                        }
                    } else {
                        println!("{:#?}", event);
                    }
                }
            }

            Ok(())
        })
        .await
}
