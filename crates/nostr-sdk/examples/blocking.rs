// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::client::blocking::Client;
use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

fn main() -> Result<()> {
    env_logger::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let client = Client::new(&my_keys);
    client.add_relay("wss://relay.nostr.info", None)?;
    client.add_relay("wss://relay.damus.io", None)?;
    client.add_relay("wss://nostr.openchain.fr", None)?;

    client.connect();

    client.delete_event(
        EventId::from_hex("57689882a98ac4db67933196c121489dea7e1231f7c0f20accad4de838500edc")?,
        Some("reason"),
    )?;

    let entity: Entity =
        client.get_entity_of("25e5c82273a271cb1a840d0060391a0bf4965cafeb029d5ab55350b418953fbb")?;
    println!("Entity: {:?}", entity);

    let subscription = SubscriptionFilter::new()
        .pubkey(my_keys.public_key())
        .since(Timestamp::now());

    client.subscribe(vec![subscription])?;

    client.disconnect_relay("wss://relay.nostr.info")?;

    client.handle_notifications(|notification| {
        if let RelayPoolNotification::Event(_url, event) = notification {
            if event.kind == Kind::EncryptedDirectMessage {
                if let Ok(msg) = decrypt(
                    &my_keys.secret_key().unwrap(),
                    &event.pubkey,
                    &event.content,
                ) {
                    println!("New DM: {}", msg);
                } else {
                    log::error!("Impossible to decrypt direct message");
                }
            } else {
                println!("{:#?}", event);
            }
        }

        Ok(())
    })?;

    Ok(())
}
