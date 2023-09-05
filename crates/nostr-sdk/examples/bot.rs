// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec12kcgs78l06p30jz7z7h3n2x2cy99nw2z6zspjdp7qc206887mwvs95lnkx";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let keys = Keys::new(secret_key);
    let opts = Options::new()
        .wait_for_connection(true)
        .skip_disconnected_relays(true)
        .send_timeout(Some(Duration::from_secs(5)));
    let client = Client::with_opts(&keys, opts);

    println!("Bot public key: {}", keys.public_key().to_bech32()?);

    client.add_relay("wss://nostr.oxtr.dev", None).await?;
    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nostr.mom", None).await?;
    client.add_relay("wss://nostr.wine", None).await?;
    client.add_relay("wss://relay.nostr.info", None).await?;

    client.connect().await;

    let metadata = Metadata::new()
        .name("nostr-sdk-bot-example")
        .display_name("Nostr SDK Bot Example")
        .website(Url::parse("https://github.com/rust-nostr/nostr")?);
    client.set_metadata(metadata).await?;

    let subscription = Filter::new()
        .pubkey(keys.public_key())
        .kind(Kind::EncryptedDirectMessage)
        .since(Timestamp::now());

    client.subscribe(vec![subscription]).await;

    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event(_url, event) = notification {
                if event.kind == Kind::EncryptedDirectMessage {
                    match nip04::decrypt(&keys.secret_key()?, &event.pubkey, &event.content) {
                        Ok(msg) => {
                            let content: String = match msg.as_str() {
                                "/rand" => rand::random::<u16>().to_string(),
                                "/help" => help(),
                                _ => {
                                    String::from("Invalid command, send /help to see all commands.")
                                }
                            };

                            client.send_direct_msg(event.pubkey, content, None).await?;
                        }
                        Err(e) => tracing::error!("Impossible to decrypt direct message: {e}"),
                    }
                }
            }
            Ok(false) // Set to true to exit from the loop
        })
        .await?;

    Ok(())
}

fn help() -> String {
    let mut output = String::new();
    output.push_str("Commands:\n");
    output.push_str("/rand - Random number\n");
    output.push_str("/help - Help");
    output
}
