use nostr::prelude::*;

pub fn run() -> Result<()> {
    let keys = Keys::vanity(vec!["0000", "yuk", "yuk0"], true, 8)?;
    println!("Secret key: {}", keys.secret_key()?.to_bech32()?);
    println!("Public key: {}", keys.public_key().to_bech32()?);
    Ok(())
}
