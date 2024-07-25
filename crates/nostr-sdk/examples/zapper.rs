// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::str::FromStr;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut nwc_uri_string = String::new();

    println!("Please enter a NWC string");
    std::io::stdin()
        .read_line(&mut nwc_uri_string)
        .expect("Failed to read line");

    // Parse NWC URI and compose NWC client
    let uri = NostrWalletConnectURI::from_str(&nwc_uri_string).expect("Failed to parse NWC URI");
    let nwc = NWC::new(uri);

    // Compose client
    let secret_key =
        SecretKey::from_bech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    let keys = Keys::new(secret_key);
    let client = Client::builder().signer(keys).zapper(nwc).build();

    client.add_relay("wss://relay.nostr.band").await?;
    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    let public_key =
        PublicKey::from_bech32("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
            .unwrap();

    // Send sats without zap event
    client.zap(&public_key, 1000, None).await?;

    // Zap profile
    let details = ZapDetails::new(ZapType::Public).message("Test");
    client.zap(public_key, 1000, Some(details)).await?;

    // Zap event
    let event_id = Nip19Event::from_bech32("nevent1qqsr0q447ylm3y3tvw07vt69w3kzk026vl6yn3dwm9fweay0dw0jttgpz3mhxue69uhhyetvv9ujumn0wd68ytnzvupzq6xcz9jerqgqkldy8lpg7lglcyj4g3nwzy2cs6u70wejdaj7csnjqvzqqqqqqygequ53").unwrap();
    let details = ZapDetails::new(ZapType::Anonymous).message("Anonymous Zap!");
    client.zap(event_id, 1000, Some(details)).await?;

    Ok(())
}
