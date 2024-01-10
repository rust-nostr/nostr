use nostr::prelude::*;

pub fn keys() -> Result<()> {
    let keys = Keys::generate();

    let public_key = keys.public_key();
    let secret_key = keys.secret_key()?;

    println!("Public key (hex): {}", public_key);
    println!("Public key (bech32): {}", public_key.to_bech32()?);
    println!("Secret key (hex): {}", keys.secret_key()?.display_secret());
    println!("Secret key (bech32): {}", secret_key.to_bech32()?);

    Ok(())
}
