// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate nostr_sdk;

use anyhow::Result;
use nostr::key::{FromBech32, Keys};
use nostr::util::nips::nip04::decrypt;
use nostr::util::time::timestamp;
use nostr::{Kind, KindBase, SubscriptionFilter};
use nostr_sdk::client::blocking::Client;
use nostr_sdk::RelayPoolNotifications;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

fn main() -> Result<()> {
    env_logger::init();

    let my_keys = Keys::from_bech32(BECH32_SK)?;

    let mut client = Client::new(&my_keys);
    client.add_relay("wss://relay.nostr.info", None)?;
    client.add_relay("wss://relay.damus.io", None)?;
    client.add_relay("wss://nostr.openchain.fr", None)?;

    client.connect()?;

    client.delete_event("57689882a98ac4db67933196c121489dea7e1231f7c0f20accad4de838500edc")?;

    let subscription = SubscriptionFilter::new()
        .pubkey(my_keys.public_key())
        .since(timestamp());

    client.subscribe(vec![subscription])?;

    client.disconnect_relay("wss://relay.nostr.info")?;

    client.handle_notifications(|notification| {
        if let RelayPoolNotifications::ReceivedEvent(event) = notification {
            if event.kind == Kind::Base(KindBase::EncryptedDirectMessage) {
                if let Ok(msg) = decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content) {
                    println!("New DM: {}", msg);
                } else {
                    log::error!("Impossible to decrypt direct message");
                }
            } else {
                println!("{:#?}", event);
            }
        }

        Ok(())
    })
}
