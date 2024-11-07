// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_connect::prelude::*;

#[derive(Debug, Clone)]
struct MyAuthUrlHandler;

#[async_trait::async_trait]
impl AuthUrlHandler for MyAuthUrlHandler {
    async fn on_auth_url(&self, auth_url: Url) -> Result<()> {
        println!("Opening auth url: {auth_url}");
        webbrowser::open(auth_url.as_str())?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let uri = NostrConnectURI::parse("bunker://79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3?relay=wss://relay.nsec.app")?;
    let app_keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let timeout = Duration::from_secs(120);

    let mut connect = NostrConnect::new(uri, app_keys, timeout, None)?;

    // Set auth_url handler
    connect.auth_url_handler(MyAuthUrlHandler);

    let receiver =
        PublicKey::parse("npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c")?;
    let content = connect.nip44_encrypt(&receiver, "Hi").await?;
    println!("Content: {content}");

    let event = EventBuilder::text_note("Testing rust-nostr", [])
        .sign(&connect)
        .await?;
    println!("Event: {}", event.as_json());

    Ok(())
}
