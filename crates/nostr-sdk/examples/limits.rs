// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Customize relay limits
    let mut limits = RelayLimits::default();
    limits.messages.max_size = Some(10_000);
    limits.events.max_size = Some(3_000);

    // OR, disable all limits
    let limits = RelayLimits::disable();

    // Compose options and limits
    let opts = Options::new().relay_limits(limits);
    let client = Client::builder().opts(opts).build();

    // Add relays and connect
    client
        .add_relays(["wss://nostr.oxtr.dev", "wss://relay.damus.io"])
        .await?;
    client.connect().await;

    // ...

    Ok(())
}
