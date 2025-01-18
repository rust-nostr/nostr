// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Account 1
    let keys1 = Keys::parse("nsec12kcgs78l06p30jz7z7h3n2x2cy99nw2z6zspjdp7qc206887mwvs95lnkx")?;
    let client = Client::new(keys1.clone());

    client.add_relay("wss://relay.damus.io").await?;
    client.connect().await;

    // Subscribe
    let filter = Filter::new()
        .author(keys1.public_key)
        .kind(Kind::TextNote)
        .limit(10);
    client.subscribe(vec![filter], None).await?;

    // Wait a little
    tokio::time::sleep(Duration::from_secs(20)).await;

    println!("Switching account...");

    // Reset client to change account
    client.reset().await;

    // Account 2
    let keys2 = Keys::parse("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")?;
    client.set_signer(keys2.clone()).await;

    client.add_relay("wss://nostr.oxtr.dev").await?;
    client.connect().await;

    println!("Account switched");

    // Subscribe
    let filter = Filter::new()
        .author(keys2.public_key)
        .kind(Kind::TextNote)
        .limit(5);
    client.subscribe(vec![filter], None).await?;

    client
        .handle_notifications(|notification| async move {
            println!("{notification:?}");
            Ok(false)
        })
        .await?;

    Ok(())
}
