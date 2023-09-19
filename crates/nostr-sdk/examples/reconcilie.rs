// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let client = Client::new(&my_keys);
    client.add_relay("wss://atl.purplerelay.com", None).await?;

    client.connect().await;

    let filter = Filter::new()
        .author(my_keys.public_key().to_string())
        .limit(10);
    let relay = client.relay("wss://atl.purplerelay.com").await?;
    relay.reconcilie(filter, Vec::new()).await?;

    Ok(())
}
