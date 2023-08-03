// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use async_utility::thread;
use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);
    let opts = Options::new().wait_for_send(false);
    let client = Client::with_opts(&my_keys, opts);

    client.add_relay("wss://nostr.oxtr.dev", None).await?;
    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nostr.openchain.fr", None).await?;

    client.connect().await;

    let subscription = Filter::new()
        .pubkey(my_keys.public_key())
        .since(Timestamp::now());

    client.subscribe(vec![subscription]).await;

    // Handle subscription notifications with `handle_notifications` method
    loop {
        client
            .handle_notifications(|notification| async {
                if let RelayPoolNotification::Event(_url, event) = notification {
                    if event.kind == Kind::EncryptedDirectMessage {
                        match decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content) {
                            Ok(msg) => {
                                let content: String = match msg.as_str() {
                                    "/stop" => match client.stop().await {
                                        Ok(_) => {
                                            let client = client.clone();
                                            thread::spawn(async move {
                                                thread::sleep(Duration::from_secs(30)).await;
                                                client.start().await;
                                                client
                                                    .send_direct_msg(
                                                        event.pubkey,
                                                        "Client restarted",
                                                        None,
                                                    )
                                                    .await
                                                    .unwrap();
                                            });
                                            String::from("Client stopped. Restaring it in 10 secs")
                                        }
                                        Err(e) => e.to_string(),
                                    },
                                    _ => String::from("Invalid command."),
                                };

                                client.send_direct_msg(event.pubkey, content, None).await?;
                            }
                            Err(e) => tracing::error!("Impossible to decrypt direct message: {e}"),
                        }
                    } else {
                        println!("{:?}", event);
                    }
                }
                Ok(false) // Set to true to exit from the loop
            })
            .await?;
    }
}
