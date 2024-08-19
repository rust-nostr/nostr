// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Init client
    let client = Client::default();
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    // Mute public key
    let muted_public_key =
        PublicKey::from_bech32("npub1l2vyh47mk2p0qlsku7hg0vn29faehy9hy34ygaclpn66ukqp3afqutajft")?;
    client.mute_public_keys([muted_public_key]).await;

    // Get events from all connected relays
    let public_key =
        PublicKey::from_bech32("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")?;
    let filter = Filter::new()
        .authors([muted_public_key, public_key])
        .kind(Kind::Metadata);
    let events = client.get_events_of(vec![filter], None).await?;
    println!("Received {} events.", events.len());

    Ok(())
}
