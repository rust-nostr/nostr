// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    let pubkey =
        PublicKey::from_hex("3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d")?;
    let profile =
        Nip19Profile::new(pubkey, vec!["wss://r.x.com", "wss://djbas.sadkb.com"]).unwrap();
    println!("{}", profile.to_bech32()?);

    Ok(())
}
