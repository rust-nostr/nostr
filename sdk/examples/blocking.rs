use std::collections::HashMap;
use std::time::Duration;

use nostr_sdk::prelude::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client: Client = Client::default();

    client
        .add_relay("wss://relay.damus.io")
        .and_connect()
        .blocking()?;

    client
        .add_relay("wss://relay.nostr.band")
        .and_connect()
        .blocking()?;

    let public_key =
        PublicKey::from_bech32("npub1080l37pfvdpyuzasyuy2ytjykjvq3ylr5jlqlg7tvzjrh9r8vn3sf5yaph")?;

    // Example 1: Subscribe with a single filter (broadcast to all relays)
    let filter = Filter::new().author(public_key).kind(Kind::Metadata);
    let output = client.subscribe(filter).blocking()?;
    println!("{:?}", output);

    // Example 2: Subscribe with multiple filters
    let filters = vec![
        Filter::new().author(public_key).kind(Kind::TextNote),
        Filter::new().author(public_key).kind(Kind::Metadata),
    ];
    let output = client.subscribe(filters).blocking()?;
    println!("{:?}", output);

    // Example 3: Targeted subscription with HashMap<&str, Filter>
    let mut targets = HashMap::new();
    targets.insert("wss://relay.damus.io", Filter::new().kind(Kind::TextNote));
    targets.insert("wss://relay.nostr.band", Filter::new().kind(Kind::Metadata));
    let output = client.subscribe(targets).blocking()?;
    println!("{:?}", output);

    // Example 4: Targeted subscription with HashMap<String, Vec<Filter>>
    let mut targets = HashMap::new();
    targets.insert(
        "wss://relay.damus.io".to_string(),
        vec![Filter::new().kind(Kind::TextNote)],
    );
    let output = client.subscribe(targets).blocking()?;

    println!("{:?}", output);

    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
