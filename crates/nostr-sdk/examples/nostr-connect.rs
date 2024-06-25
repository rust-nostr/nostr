// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

const APP_SECRET_KEY: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let app_keys = Keys::parse(APP_SECRET_KEY)?;

    // Compose signer from bunker URI
    let uri = NostrConnectURI::parse("bunker://79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3?relay=wss%3A%2F%2Frelay.nsec.app")?;
    let signer = Nip46Signer::new(uri, app_keys, Duration::from_secs(60), None).await?;

    // Compose signer
    /* let uri = NostrConnectURI::client(
        app_keys.public_key(),
        [Url::parse("wss://relay.nsec.app")?],
        "Test app",
    );
    println!("\n{uri}\n");
    let signer = Nip46Signer::new(uri, app_keys, Duration::from_secs(60), None).await?; */

    // Get bunker URI for future connections
    //let bunker_uri = signer.nostr_connect_uri().await;

    // Compose client
    let client = Client::new(signer);
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    // Publish events
    let output = client
        .publish_text_note("Testing rust-nostr NIP46 signer [bunker]", [])
        .await?;
    println!("Published text note: {}\n", output.val);

    let receiver =
        PublicKey::from_bech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;
    let output = client
        .send_private_msg(receiver, "Hello from rust-nostr", None)
        .await?;
    println!("Sent DM: {}", output.val);

    Ok(())
}
