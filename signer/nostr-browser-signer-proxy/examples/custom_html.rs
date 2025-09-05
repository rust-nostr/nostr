// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_browser_signer_proxy::prelude::*;
use tokio::{signal, time};

const CUSTOM_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NIP07 Signer Proxy</title>
    <script src="proxy.js"></script>
</head>
<body>
    <h1>NIP07 Signer Proxy</h1>
    <p>Proxy Status: <span id="nip07-proxy-status">Loading...</span></p>
</body>
</html>
"#;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Use `include_str!` macro in your code.
    let options = BrowserSignerProxyOptions::default().custom_html_page(CUSTOM_HTML);
    let proxy = BrowserSignerProxy::new(options);

    proxy.start().await?;

    println!("Url: {}", proxy.url());

    // Waits until the proxy session becomes active, checking every second.
    loop {
        if proxy.is_session_active() {
            break;
        }
        time::sleep(Duration::from_secs(1)).await;
    }

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
