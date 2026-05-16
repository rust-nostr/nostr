// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const EXAMPLE_SNIPPET: &str = include_str!("code_snippet.rs");

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nos.lol").await?;
    client.add_relay("wss://nostr.mom").await?;

    client.connect().await;

    let keys = Keys::generate();

    // Build a code snippet for this example :)
    let snippet: CodeSnippet = CodeSnippet::new(EXAMPLE_SNIPPET)
        .name("code_snippts.rs")
        .description("Snippet that snippet itself")
        .language("rust")
        .extension("rs")
        .license("MIT");

    let event = EventBuilder::code_snippet(snippet).finalize(&keys)?;

    let output = client.send_event(&event).await?;

    let nevent = Nip19Event::new(*output.id()).relays(vec![
        RelayUrl::parse("wss://nos.lol")?,
        RelayUrl::parse("wss://nostr.mom")?,
    ]);

    tracing::info!("Done, check the event `{}`", nevent.to_bech32()?);

    client.shutdown().await;

    Ok(())
}
