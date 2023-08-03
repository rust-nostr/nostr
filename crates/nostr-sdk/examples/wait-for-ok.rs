// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let opts = Options::new()
        .wait_for_ok(true)
        .send_timeout(Some(Duration::from_secs(30)));
    let client = Client::with_opts(&my_keys, opts);

    // Paid relays that will not allow this public key to publish event
    client.add_relay("wss://eden.nostr.land", None).await?;
    client.add_relay("wss://nostr.wine", None).await?;

    client.connect().await;

    // Publish a text note
    client
        .publish_text_note("Hello from rust nostr-sdk", &[])
        .await?;

    Ok(())
}
