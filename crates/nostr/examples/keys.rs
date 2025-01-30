// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    // Random keys
    let keys = Keys::generate();
    let public_key = keys.public_key();
    let secret_key = keys.secret_key();

    println!("Public key: {}", public_key);
    println!("Public key bech32: {}", public_key.to_bech32()?);
    println!("Secret key: {}", secret_key.to_secret_hex());
    println!("Secret key bech32: {}", secret_key.to_bech32()?);

    // Bech32 keys
    let keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    println!("Public key: {}", keys.public_key());

    Ok(())
}
