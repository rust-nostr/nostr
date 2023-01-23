// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let opts = Options::new().wait_for_send(true);

    let client = Client::new_with_opts(&my_keys, opts);
    client.add_relay("wss://relay.nostr.info", None).await?;
    client.add_relay("wss://relay.damus.io", None).await?;

    client.connect().await;

    client.publish_text_note("Hello world", &[]).await?;

    Ok(())
}
