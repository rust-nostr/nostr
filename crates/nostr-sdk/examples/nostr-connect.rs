// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

const APP_SECRET_KEY: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Compose signer
    let secret_key = SecretKey::from_bech32(APP_SECRET_KEY)?;
    let app_keys = Keys::new(secret_key);
    let relay_url = Url::parse("wss://relay.rip")?;
    let signer = Nip46Signer::new(relay_url, app_keys, None, Duration::from_secs(60)).await?;

    // Compose URI
    let metadata = NostrConnectMetadata::new("Nostr SDK").url(Url::parse("https://example.com")?);
    let nostr_connect_uri: NostrConnectURI = signer.nostr_connect_uri(metadata);

    println!("\n###############################################\n");
    println!("Nostr Connect URI: {nostr_connect_uri}");
    println!("\n###############################################\n");

    // Create client
    let client = Client::new(signer);
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    let id = client
        .publish_text_note("Testing nostr-sdk nostr-connect client", [])
        .await?;
    println!("Published text note: {id}\n");

    let receiver = XOnlyPublicKey::from_bech32(
        "npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
    )?;
    client
        .send_direct_msg(receiver, "Hello from nostr-sdk", None)
        .await?;
    println!("Sent DM: {id}");

    Ok(())
}
