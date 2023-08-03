// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::prelude::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let pubkey = XOnlyPublicKey::from_str(
        "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d",
    )?;
    let profile = Profile::new(pubkey, vec!["wss://r.x.com", "wss://djbas.sadkb.com"]);
    println!("{}", profile.to_bech32()?);

    Ok(())
}
