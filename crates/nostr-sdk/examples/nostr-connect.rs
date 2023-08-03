// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

const APP_SECRET_KEY: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(APP_SECRET_KEY)?;
    let app_keys = Keys::new(secret_key);
    let relay_url = Url::parse("wss://relay.damus.io")?;
    let signer = RemoteSigner::new(relay_url.clone(), None);

    let client = Client::with_remote_signer(&app_keys, signer);
    client.add_relay(relay_url, None).await?;

    let metadata = NostrConnectMetadata::new("Nostr SDK").url(Url::parse("https://example.com")?);
    let nostr_connect_uri: NostrConnectURI = client.nostr_connect_uri(metadata)?;

    println!("\n###############################################\n");
    println!("Nostr Connect URI: {nostr_connect_uri}");
    println!("\n###############################################\n");

    client.connect().await;

    // Request signer public key since we not added in Client::with_remote_signer
    client
        .req_signer_public_key(Some(Duration::from_secs(180)))
        .await?;

    let id = client
        .publish_text_note("Testing nostr-sdk nostr-connect client", &[])
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
