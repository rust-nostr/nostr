// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_gossip_memory::prelude::*;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::parse("nsec12kcgs78l06p30jz7z7h3n2x2cy99nw2z6zspjdp7qc206887mwvs95lnkx")?;
    let gossip = NostrGossipMemory::unbounded();
    let client = Client::builder()
        .signer(keys.clone())
        .gossip(gossip)
        .build();

    println!("Bot public key: {}", keys.public_key().to_bech32()?);

    client.add_relay("wss://nostr.oxtr.dev").await?;
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.mom").await?;
    client.add_relay("wss://nostr.wine").await?;
    client.add_relay("wss://relay.nostr.info").await?;
    client.add_relay("wss://auth.nostr1.com").await?;

    client.connect().await;

    let subscription = Filter::new()
        .pubkey(keys.public_key())
        .kind(Kind::GiftWrap)
        .limit(0); // Limit set to 0 to get only new events! Timestamp::now() CAN'T be used for gift wrap since the timestamps are tweaked!

    client.subscribe(subscription).await?;

    client
        .handle_notifications(|notification| async {
            if let ClientNotification::Event { event, .. } = notification {
                if event.kind == Kind::GiftWrap {
                    match UnwrappedGift::from_gift_wrap(&keys, &event).await {
                        Ok(UnwrappedGift { rumor, sender }) => {
                            if rumor.kind == Kind::PrivateDirectMessage {
                                let content: String = match rumor.content.as_str() {
                                    "/hello" => String::from("Hello, World!"),
                                    "/help" => help(),
                                    _ => String::from(
                                        "Invalid command, send /help to see all commands.",
                                    ),
                                };

                                // Send private message
                                let msg =
                                    EventBuilder::private_msg(&keys, sender, content, []).await?;
                                client.send_event(&msg).to_nip17().await?;
                            }
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
    output.push_str("/hello - Hello\n");
    output.push_str("/help - Help");
    output
}
