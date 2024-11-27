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
    let builder = EventBuilder::text_note("Hello, rust-nostr!");
    let output = client.send_event_builder(builder).await?;
    // ANCHOR_END: publish

    // ANCHOR: output
    println!("Event ID: {}", output.id().to_bech32()?);
    println!("Sent to: {:?}", output.success);
    println!("Not sent to: {:?}", output.failed);
    // ANCHOR_END: output

    Ok(())
}

// ANCHOR_END: full
