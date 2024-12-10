// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Init client
    let opts = Options::new().filtering_mode(RelayFilteringMode::Whitelist);
    let client = Client::builder().opts(opts).build();
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    let not_in_whitelist_public_key =
        PublicKey::from_bech32("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")?;

    // Allowed public key
    let allowed_public_key =
        PublicKey::from_bech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;
    let filtering = client.filtering();
    filtering.add_public_keys([allowed_public_key]).await;

    // Get events from all connected relays
    let filter = Filter::new()
        .authors([allowed_public_key, not_in_whitelist_public_key])
        .kind(Kind::Metadata);
    let events = client
        .fetch_events(vec![filter], Duration::from_secs(10))
        .await?;
    println!("Received {} events.", events.len());

    for event in events.into_iter() {
        println!("{}", event.as_json());
    }

    Ok(())
}
