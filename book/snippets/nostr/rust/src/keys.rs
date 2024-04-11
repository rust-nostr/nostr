use nostr::prelude::*;

// ANCHOR: generate
pub fn generate() -> Result<()> {
    let keys = Keys::generate();

    let public_key = keys.public_key();
    let secret_key = keys.secret_key()?;

    println!("Public key (hex): {}", public_key);
    println!("Public key (bech32): {}", public_key.to_bech32()?);
    println!("Secret key (hex): {}", keys.secret_key()?.to_secret_hex());
    println!("Secret key (bech32): {}", secret_key.to_bech32()?);
    
    Ok(())
}
// ANCHOR_END: generate

// ANCHOR: restore
pub fn restore() -> Result<()> {
    // Parse keys directly from secret key
    let keys = Keys::parse("secret-key")?;
    
    // Parse secret key and construct keys
    let secret_key = SecretKey::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")?;
    let keys = Keys::new(secret_key);

    // Restore from bech32
    let secret_key = SecretKey::from_bech32("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;
    let keys = Keys::new(secret_key);

    // Restore from hex
    let secret_key = SecretKey::from_hex("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")?;
    let keys = Keys::new(secret_key);

    Ok(())
}
// ANCHOR_END: restore

// ANCHOR: vanity
pub fn vanity() -> Result<()> {
    let keys = Keys::vanity(vec!["0000", "yuk", "yuk0"], true, 8)?;
    println!("Secret key: {}", keys.secret_key()?.to_bech32()?);
    println!("Public key: {}", keys.public_key().to_bech32()?);
    Ok(())
}
// ANCHOR_END: vanity
