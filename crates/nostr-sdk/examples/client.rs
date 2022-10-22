// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

extern crate nostr_sdk;

use anyhow::Result;
use chrono::Utc;
use nostr_sdk::{Client, RelayPoolNotifications};
use nostr_sdk_base::util::nip04::decrypt;
use nostr_sdk_base::{Keys, Kind, KindBase, SubscriptionFilter};

const BECH32_SK: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let my_keys = Keys::new_from_bech32(BECH32_SK)?;

    let client = Client::new(&my_keys, None);
    client.add_relay("ws://localhost:8090").await?;
    client.add_relay("wss://relay.damus.io").await?;

    client.connect_all().await;

    client
        .delete_event("57689882a98ac4db67933196c121489dea7e1231f7c0f20accad4de838500edc")
        .await?;

    let subscription = SubscriptionFilter::new()
        .pubkey(my_keys.public_key)
        .since(Utc::now());

    client.subscribe(vec![subscription]).await;

    client
        .keep_alive(|notification| {
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
                RelayPoolNotifications::RelayDisconnected(url) => {
                    log::warn!("Relay {} disconnected", url);
                }
            }

            Ok(())
        })
        .await
}
