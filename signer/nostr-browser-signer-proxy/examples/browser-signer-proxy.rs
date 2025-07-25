// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_browser_signer_proxy::prelude::*;
use tokio::{signal, time};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let proxy = BrowserSignerProxy::new(BrowserSignerProxyOptions::default());

    proxy.start().await?;

    println!("Url: {}", proxy.url());

    // Give time to open the webpage
    time::sleep(Duration::from_secs(10)).await;

    // Get public key
    let public_key = proxy.get_public_key().await?;
    println!("Public key: {}", public_key);

    // Sign event
    let event = EventBuilder::text_note("Testing browser signer proxy")
        .sign(&proxy)
        .await?;
    println!("Event: {}", event.as_json());

    // Build a gift wrap
    let receiver =
        PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;
    let rumor = EventBuilder::new(Kind::Custom(123), "test").build(public_key);
    let gift_wrap = EventBuilder::gift_wrap(&proxy, &receiver, rumor, []).await?;
    println!("Gift wrap: {}", gift_wrap.as_json());

    // Keep up the program
    signal::ctrl_c().await?;

    Ok(())
}
