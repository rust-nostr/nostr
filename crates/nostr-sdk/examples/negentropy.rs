// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let database = NostrLMDB::open("./db/nostr-lmdb")?;
    let client: Client = ClientBuilder::default().database(database).build();

    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://nostr.oxtr.dev").await?;

    client.connect().await;

    let public_key =
        PublicKey::from_bech32("npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s")?;
    let filter = Filter::new().author(public_key);
    let (tx, mut rx) = SyncProgress::channel();
    let opts = SyncOptions::default().progress(tx);

    tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            let SyncProgress { total, current } = *rx.borrow_and_update();
            if total > 0 {
                println!("{:.2}%", (current as f64 / total as f64) * 100.0);
            }
        }
    });

    let output = client.sync(filter, opts).await?;
    println!("Success: {:?}", output.success);
    println!("Failed: {:?}", output.failed);

    Ok(())
}
