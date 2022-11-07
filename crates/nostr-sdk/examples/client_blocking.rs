// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate nostr_sdk;

use anyhow::Result;
use chrono::Utc;
use nostr_sdk::{Client, RelayPoolNotifications};
use nostr_sdk_base::util::nip04::decrypt;
use nostr_sdk_base::{Keys, Kind, KindBase, SubscriptionFilter};

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

fn main() -> Result<()> {
    env_logger::init();

    let my_keys = Keys::new_from_bech32(BECH32_SK)?;

    let mut client = Client::new(&my_keys, None);
    client.add_relay("wss://relay.nostr.info", None)?;
    client.add_relay("wss://relay.damus.io", None)?;
    client.add_relay("wss://nostr.openchain.fr", None)?;

    client.connect_all()?;

    client.delete_event("57689882a98ac4db67933196c121489dea7e1231f7c0f20accad4de838500edc")?;

    let subscription = SubscriptionFilter::new()
        .pubkey(my_keys.public_key)
        .since(Utc::now());

    client.subscribe(vec![subscription])?;

    client.disconnect_relay("wss://relay.nostr.info")?;

    client.handle_notifications(|notification| {
        match notification {
            RelayPoolNotifications::ReceivedEvent(event) => {
                if event.kind == Kind::Base(KindBase::EncryptedDirectMessage) {
                    if let Ok(msg) = decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content)
                    {
                        println!("New DM: {}", msg);
                    } else {
                        log::error!("Impossible to decrypt direct message");
                    }
                } else {
                    println!("{:#?}", event);
                }
            }
            RelayPoolNotifications::RelayDisconnected(url) => {
                log::warn!("Relay {} disconnected", url);
            }
        }

        Ok(())
    })
}
