// ANCHOR: full
use nostr_sdk::prelude::*;

pub async fn hello() -> Result<()> {
    // ANCHOR: client
    let keys: Keys = Keys::generate();
    let client = Client::new(keys);
    // ANCHOR_END: client

    // ANCHOR: connect
    client.add_relay("wss://relay.damus.io").await?;

    client.connect().await;
    // ANCHOR_END: connect

    // ANCHOR: publish
    let builder = EventBuilder::text_note("Hello, rust-nostr!", []);
    client.send_event_builder(builder).await?;
    // ANCHOR_END: publish

    Ok(())
}

// ANCHOR_END: full
