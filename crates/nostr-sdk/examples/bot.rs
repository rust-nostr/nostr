// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec12kcgs78l06p30jz7z7h3n2x2cy99nw2z6zspjdp7qc206887mwvs95lnkx";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let keys = Keys::new(secret_key);
    let opts = Options::new().automatic_decryption(true);
    let client = Client::with_opts(&keys, opts);

    println!("Bot public key: {}", keys.public_key().to_bech32()?);

    client.add_relay("wss://nostr.oxtr.dev").await?;
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.mom").await?;
    client.add_relay("wss://nostr.wine").await?;

    client.connect().await;

    let metadata = Metadata::new()
        .name("rust-nostr-bot-example")
        .display_name("rust-nostr bot example")
        .website(Url::parse("https://github.com/rust-nostr/nostr")?);
    client.set_metadata(&metadata).await?;

    let subscription = Filter::new()
        .pubkey(keys.public_key())
        .kind(Kind::GiftWrap)
        .limit(0); // Limit set to 0 to get only new events! Timestamp::now() CAN'T be used for gift wrap since the timestamps are tweaked!

    client.subscribe(vec![subscription], None).await?;

    client
        .handle_notifications(|notification| async {
            if let ClientNotification::PrivateDirectMessage {
                sender,
                message,
                timestamp,
                ..
            } = notification
            {
                println!(
                    "Received a private direct message from {sender}: {message} [{timestamp}]"
                );
                let content: String = match message.as_str() {
                    "/rand" => rand::random::<u16>().to_string(),
                    "/help" => help(),
                    _ => String::from("Invalid command, send /help to see all commands."),
                };
                client.send_private_msg(sender, content, None).await?;
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
