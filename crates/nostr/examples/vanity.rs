// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    let num_cores = num_cpus::get();
    let keys = Keys::vanity(vec!["0000", "yuk", "yuk0"], true, num_cores)?;
    println!("Secret key: {}", keys.secret_key()?.to_bech32()?);
    println!("Public key: {}", keys.public_key().to_bech32()?);
    Ok(())
}
