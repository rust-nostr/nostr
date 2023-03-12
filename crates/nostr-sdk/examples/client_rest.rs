// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::client::rest::Client;
use nostr_sdk::prelude::*;

const BECH32_SK: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let secret_key = SecretKey::from_bech32(BECH32_SK)?;
    let my_keys = Keys::new(secret_key);

    let url = Url::parse("http://127.0.0.1:7773")?;
    let client = Client::new(&my_keys, url);

    let contacts = client.get_contact_list_metadata().await?;
    println!("{contacts:?}");

    client
        .publish_text_note("Hello from nostr-sdk WASM!", &[])
        .await?;

    let filter = Filter::new().author(my_keys.public_key());
    let events = client.get_events_of(vec![filter]).await?;

    println!("{events:?}");

    Ok(())
}
