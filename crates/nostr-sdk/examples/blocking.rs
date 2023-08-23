// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::client::blocking::Client;
use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);
    let opts = Options::new().wait_for_send(false);
    let client = Client::with_opts(&my_keys, opts);

    client.add_relay("wss://relay.nostr.info", None)?;
    client.add_relay("wss://relay.damus.io", None)?;
    client.add_relay("wss://nostr.openchain.fr", None)?;

    client.connect();

    client.delete_event(
        EventId::from_hex("57689882a98ac4db67933196c121489dea7e1231f7c0f20accad4de838500edc")?,
        Some("reason"),
    )?;

    let subscription = Filter::new()
        .pubkey(my_keys.public_key())
        .since(Timestamp::now());

    client.subscribe(vec![subscription]);

    client.disconnect_relay("wss://relay.nostr.info")?;

    client.handle_notifications(|notification| {
        if let RelayPoolNotification::Event(_url, event) = notification {
            if event.kind == Kind::EncryptedDirectMessage {
                if let Ok(msg) = nip04::decrypt(
                    &my_keys.secret_key().unwrap(),
                    &event.pubkey,
                    &event.content,
                ) {
                    println!("New DM: {}", msg);
                    // return Ok(true);
                } else {
                    tracing::error!("Impossible to decrypt direct message");
                }
            } else {
                println!("{:#?}", event);
            }
        }

        Ok(false) // Set to true to exit from the loop
    })?;

    Ok(())
}
