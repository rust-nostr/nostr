// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let public_key =
        PublicKey::from_bech32("npub1080l37pfvdpyuzasyuy2ytjykjvq3ylr5jlqlg7tvzjrh9r8vn3sf5yaph")?;

    let client = Client::default();
    client.add_relay("wss://atl.purplerelay.com").await?;

    client.connect().await;

    let my_items = Vec::new();
    let filter = Filter::new().author(public_key).limit(10);
    let relay = client.relay("wss://atl.purplerelay.com").await?;
    let opts = NegentropyOptions::default();
    relay.reconcile(filter, my_items, opts).await?;

    Ok(())
}
