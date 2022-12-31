// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::util::nips::nip19::{Profile, ToBech32};
use nostr::Result;

fn main() -> Result<()> {
    env_logger::init();

    let pubkey = XOnlyPublicKey::from_str(
        "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d",
    )?;
    let profile = Profile::new(pubkey, vec!["wss://r.x.com", "wss://djbas.sadkb.com"]);
    println!("{}", profile.to_bech32()?);

    Ok(())
}
