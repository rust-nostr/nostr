// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let public_key = XOnlyPublicKey::from_bech32(
        "npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c",
    )?;

    let client = Client::default();
    client.add_relay("wss://nostr.mikedilger.com").await?;

    client.connect().await;

    println!("Subscribing to Relay List Metadata");
    client
        .subscribe(vec![Filter::new().author(public_key).kind(Kind::RelayList)])
        .await;

    client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind() == Kind::RelayList {
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
