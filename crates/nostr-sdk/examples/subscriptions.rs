// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

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
        .author(my_keys.public_key().to_string())
        .kind(Kind::Metadata)
        .since(Timestamp::now());

    // Subscribe using`InternalSubscriptionId::Pool`
    client.subscribe(vec![subscription]).await;

    // Subscribe using custom `InternalSubscriptionId`
    // This not overwrite the previous subscription since has a different internal ID
    let relay = client.relay("wss://relay.damus.io").await?;
    let other_filters = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .pubkey(my_keys.public_key())
        .since(Timestamp::now());
    relay
        .subscribe_with_internal_id(
            InternalSubscriptionId::Custom(String::from("other-id")),
            vec![other_filters],
            None,
        )
        .await?;

    // Handle subscription notifications with `handle_notifications` method
    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event(_url, event) = notification {
                if event.kind == Kind::EncryptedDirectMessage {
                    if nip04::decrypt(&my_keys.secret_key()?, &event.pubkey, &event.content).is_ok()
                    {
                        // Overwrite subscrption with `other-id` internal ID
                        let relay = client.relay("wss://relay.damus.io").await?;
                        let other_filters = Filter::new()
                            .kind(Kind::TextNote)
                            .author(my_keys.public_key().to_string())
                            .since(Timestamp::now());
                        relay
                            .subscribe_with_internal_id(
                                InternalSubscriptionId::Custom(String::from("other-id")),
                                vec![other_filters],
                                None,
                            )
                            .await?;
                    } else {
                        tracing::error!("Impossible to decrypt direct message");
                    }
                } else if event.kind == Kind::TextNote {
                    println!("TextNote: {:?}", event);
                    let relay = client.relay("wss://relay.damus.io").await?;
                    relay
                        .unsubscribe_with_internal_id(
                            InternalSubscriptionId::Custom(String::from("other-id")),
                            None,
                        )
                        .await?;
                    // OR
                    // relay.unsubscribe_all(None).await?;
                } else {
                    println!("{:?}", event);
                }
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}
