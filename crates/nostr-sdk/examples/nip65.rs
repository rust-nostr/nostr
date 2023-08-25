// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::Keys;
use nostr_sdk::nips::nip65;
use nostr_sdk::prelude::FromBech32;
use nostr_sdk::secp256k1::XOnlyPublicKey;
use nostr_sdk::{Client, Filter, Kind, RelayPoolNotification, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let public_key = XOnlyPublicKey::from_bech32(
        "npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c",
    )?;
    let my_keys: nostr::Keys = Keys::from_public_key(public_key);

    let client = Client::new(&my_keys);
    client.add_relay("wss://nostr.mikedilger.com", None).await?;

    client.connect().await;

    println!("Subscribing to Relay List Metadata");
    client
        .subscribe(vec![Filter::new()
            .author(public_key.to_string())
            .kind(Kind::RelayList)])
        .await;

    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event(_url, event) = notification {
                if event.kind == Kind::RelayList {
                    let list = nip65::extract_relay_list(&event);
                    println!("Found relay list metadata: {list:?}");
                    return Ok(true); // Exit from loop
                }
            }

            Ok(false)
        })
        .await?;

    Ok(())
}
