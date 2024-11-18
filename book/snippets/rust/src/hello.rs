// ANCHOR: all
use nostr_sdk::prelude::*;

pub async fn hello() -> Result<()> {
    // ANCHOR: keys
    let keys: Keys = Keys::generate();
    // ANCHOR_END: keys
    
    // ANCHOR: client
    let client = Client::new(keys);
    // ANCHOR_END: client

    // ANCHOR: connect
    client.add_relay("wss://relay.damus.io").await?;
    client.add_read_relay("wss://relay.nostr.info").await?;

    client.connect().await;
    // ANCHOR_END: connect

    // ANCHOR: publish
    let builder = EventBuilder::text_note("Hello, rust-nostr!", []);
    client.send_event_builder(builder).await?;
    // ANCHOR_END: publish

    Ok(())
}

// ANCHOR_END: all
