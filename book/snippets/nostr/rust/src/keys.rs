use nostr::prelude::*;

pub fn keys() -> Result<()> {
    let keys = Keys::generate();

    let public_key = keys.public_key();
    let secret_key = keys.secret_key()?;

    println!("Public key (hex): {}", public_key);
    println!("Public key (bech32): {}", public_key.to_bech32());
    println!("Secret key (hex): {}", keys.secret_key()?.to_secret_hex());
    println!("Secret key (bech32): {}", secret_key.to_bech32());

    // Parse keys from hex or bech32
    let keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;

    Ok(())
}
