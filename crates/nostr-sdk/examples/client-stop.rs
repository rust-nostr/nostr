// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use async_utility::thread;
use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let opts = Options::new().send_timeout(Some(Duration::from_secs(10)));
    let client = Client::with_opts(&my_keys, opts);

    client.add_relay("wss://nostr.oxtr.dev", None).await?;
    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nostr.openchain.fr", None).await?;

    client.connect().await;

    thread::sleep(Duration::from_secs(10)).await;

    client.stop().await?;

    thread::sleep(Duration::from_secs(15)).await;

    client.start().await;

    thread::sleep(Duration::from_secs(10)).await;

    client.publish_text_note("Test", &[]).await?;

    Ok(())
}
