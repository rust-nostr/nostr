// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);
    let opts = Options::new().wait_for_send(false);
    let client = Client::with_opts(&my_keys, opts);

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

    client.add_relay("wss://nostr.oxtr.dev", None).await?;
    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nostr.openchain.fr", None).await?;
    client
        .add_relay_with_opts("wss://nostr.mom", None, RelayOptions::new(true, false))
        .await?;
    client
        .add_relay(
            "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
            proxy,
        )
        .await?;

    client.connect().await;

    let subscription = Filter::new()
        .pubkey(my_keys.public_key())
        .since(Timestamp::now());

    client.subscribe(vec![subscription]).await;

    // Handle subscription notifications with `handle_notifications` method
    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event(_url, event) = notification {
                if event.kind == Kind::EncryptedDirectMessage {
                    if let Ok(msg) =
                        nip04::decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content)
                    {
                        println!("New DM: {msg}");
                        client
                            .send_direct_msg(event.pubkey, msg, Some(event.id))
                            .await?;
                    } else {
                        tracing::error!("Impossible to decrypt direct message");
                    }
                } else {
                    println!("{:?}", event);
                }
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    // Handle subscription notifications with `notifications` channel receiver
    /* let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event(_url, event) = notification {
            if event.kind == Kind::EncryptedDirectMessage {
                if let Ok(msg) = decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content) {
                    println!("New DM: {msg}");
                    client.send_direct_msg(event.pubkey, msg).await?;
                } else {
                    tracing::error!("Impossible to decrypt direct message");
                }
            } else {
                println!("{:?}", event);
            }
        }
    } */

    Ok(())
}
