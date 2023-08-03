// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::nips::nip06::{FromMnemonic, GenerateMnemonic};
use nostr::nips::nip19::ToBech32;
use nostr::{Keys, Result};

const MNEMONIC_PHRASE: &str = "equal dragon fabric refuse stable cherry smoke allow alley easy never medal attend together lumber movie what sad siege weather matrix buffalo state shoot";

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("Mnemonic: {}", Keys::generate_mnemonic(12)?);

    let keys = Keys::from_mnemonic(MNEMONIC_PHRASE, Some("mypassphrase"))?;
    println!("{}", keys.secret_key()?.to_bech32()?);

    Ok(())
}
