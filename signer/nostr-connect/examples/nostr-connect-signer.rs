// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use dialoguer::Confirm;
use nostr_connect::prelude::*;

const SIGNER_SECRET_KEY: &str = "nsec12kcgs78l06p30jz7z7h3n2x2cy99nw2z6zspjdp7qc206887mwvs95lnkx";
const USER_SECRET_KEY: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let keys = NostrConnectKeys {
        signer: Keys::parse(SIGNER_SECRET_KEY)?,
        user: Keys::parse(USER_SECRET_KEY)?,
    };

    // Compose signer
    let signer = NostrConnectRemoteSigner::new(keys, ["wss://relay.nsec.app"], None, None)?;

    // Compose signer from URI
    // let uri = NostrConnectURI::parse("nostrconnect://...")?;
    // let signer = NostrConnectRemoteSigner::from_uri(uri, keys, None, None)?;

    // Print bunker URI
    let uri = signer.bunker_uri();
    println!("\n{uri}\n");

    // Serve signer
    signer.serve(CustomActions).await?;

    Ok(())
}

struct CustomActions;

impl NostrConnectSignerActions for CustomActions {
    fn approve(&self, public_key: &PublicKey, req: &NostrConnectRequest) -> bool {
        println!("Public key: {public_key}");
        println!("{req:#?}\n");
        Confirm::new()
            .with_prompt("Approve request?")
            .default(false)
            .interact()
            .unwrap_or_default()
    }
}
