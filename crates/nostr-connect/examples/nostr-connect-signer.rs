// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use dialoguer::Confirm;
use nostr::nips::nip46::Request;
use nostr_connect::prelude::*;

const USER_SECRET_KEY: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::parse(USER_SECRET_KEY)?;

    // Compose signer
    //let signer = NostrConnectRemoteSigner::new(secret_key, ["wss://relay.rip"], None, None).await?;

    // Compose signer from URI
    let uri = NostrConnectURI::parse("nostrconnect://aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4?metadata=%7B%22name%22%3A%22Test+app%22%7D&relay=wss%3A%2F%2Frelay.nsec.app")?;
    let signer = NostrConnectRemoteSigner::from_uri(uri, secret_key, None, None).await?;

    // Print bunker URI
    let uri = signer.bunker_uri().await;
    println!("\n{uri}\n");

    // Serve signer
    signer.serve(CustomActions).await?;

    Ok(())
}

struct CustomActions;

impl NostrConnectSignerActions for CustomActions {
    fn approve(&self, req: &Request) -> bool {
        println!("{req:#?}\n");
        Confirm::new()
            .with_prompt("Approve request?")
            .default(false)
            .interact()
            .unwrap_or_default()
    }
}
