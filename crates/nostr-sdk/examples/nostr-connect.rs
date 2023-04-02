// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

const APP_SECRET_KEY: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let secret_key = SecretKey::from_bech32(APP_SECRET_KEY)?;
    let app_keys = Keys::new(secret_key);
    let relay_url = Url::parse("ws://192.168.7.233:7777")?;

    let client = Client::with_remote_signer(&app_keys, relay_url, None);
    client.add_relay("ws://192.168.7.233:7777", None).await?;

    let metadata = NostrConnectMetadata::new("Nostr SDK").url(Url::parse("https://example.com")?);
    let nostr_connect_uri: NostrConnectURI = client.nostr_connect_uri(metadata)?;

    println!("\n###############################################\n");
    println!("Nostr Connect URI: {nostr_connect_uri}");
    println!("\n###############################################\n");

    client.connect().await;

    // Init Nostr Connect client
    client
        .init_nostr_connect(Some(Duration::from_secs(180)))
        .await?;

    let id = client
        .publish_text_note("Testing nostr-sdk nostr-connect client", &[])
        .await?;
    println!("Published event {id}");

    Ok(())
}
