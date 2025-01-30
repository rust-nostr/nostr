// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    client.add_relay("wss://nostr.oxtr.dev").await?;
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.openchain.fr").await?;

    client.connect().await;

    let keys = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let public_key = keys.public_key();

    let subscription = Filter::new()
        .author(public_key)
        .kind(Kind::Metadata)
        .since(Timestamp::now());

    // Subscribe (auto generate subscription ID)
    let Output { val: sub_id_1, .. } = client.subscribe(subscription, None).await?;

    // Subscribe with custom ID
    let sub_id_2 = SubscriptionId::new("other-id");
    let filter = Filter::new()
        .author(public_key)
        .kind(Kind::TextNote)
        .since(Timestamp::now());
    client
        .subscribe_with_id(sub_id_2.clone(), filter, None)
        .await?;

    // Overwrite previous subscription
    let filter = Filter::new()
        .author(public_key)
        .kind(Kind::EncryptedDirectMessage)
        .since(Timestamp::now());
    client
        .subscribe_with_id(sub_id_1.clone(), filter, None)
        .await?;

    // Handle subscription notifications with `handle_notifications` method
    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event {
                subscription_id,
                event,
                ..
            } = notification
            {
                // Check subscription ID
                if subscription_id == sub_id_1 {
                    // Handle (ex. update specific UI)
                }

                // Check kind
                if event.kind == Kind::EncryptedDirectMessage {
                    if let Ok(msg) =
                        nip04::decrypt(keys.secret_key(), &event.pubkey, &event.content)
                    {
                        println!("DM: {msg}");
                    } else {
                        tracing::error!("Impossible to decrypt direct message");
                    }
                } else if event.kind == Kind::TextNote {
                    println!("TextNote: {:?}", event);
                } else {
                    println!("{:?}", event);
                }
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}
