// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const EXAMPLE_SNIPPET: &str = include_str!("code_snippet.rs");

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = Keys::generate();
    let client = Client::new(keys);

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nos.lol").await?;
    client.add_relay("wss://nostr.mom").await?;

    client.connect().await;

    // Build a code snippet for this example :)
    let snippet: CodeSnippet = CodeSnippet::new(EXAMPLE_SNIPPET)
        .name("code_snippts.rs")
        .description("Snippet that snippet itself")
        .language("rust")
        .extension("rs")
        .license("MIT");

    let builder = EventBuilder::code_snippet(snippet);

    let event = client.send_event_builder(builder).await?;
    let nevent = Nip19Event::new(*event.id()).relays(vec![
        RelayUrl::parse("wss://nos.lol")?,
        RelayUrl::parse("wss://nostr.mom")?,
    ]);

    tracing::info!("Done, check the event `{}`", nevent.to_bech32()?);

    client.shutdown().await;

    Ok(())
}
