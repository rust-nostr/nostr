// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    // Random keys
    let keys = Keys::generate();
    let public_key = keys.public_key();
    let secret_key = keys.secret_key()?;

    println!("Public key: {}", public_key);
    println!("Public key bech32: {}", public_key.to_bech32());
    println!("Secret key: {}", keys.secret_key()?.to_secret_hex());
    println!("Secret key bech32: {}", secret_key.to_bech32());

    // Bech32 keys
    let secret_key =
        SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let keys = Keys::new(secret_key);
    println!("Public key: {}", keys.public_key());

    let public_key =
        PublicKey::from_bech32("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")?;
    let keys = Keys::from_public_key(public_key);
    println!("Public key: {}", keys.public_key());

    Ok(())
}
