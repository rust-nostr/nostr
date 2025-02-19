// ANCHOR: full
use nostr_sdk::prelude::*;

pub fn encrypt() -> Result<()> {
    // ANCHOR: parse-secret-key
    let secret_key: SecretKey = SecretKey::parse("3501454135014541350145413501453fefb02227e449e57cf4d3a3ce05378683")?;
    // ANCHOR_END: parse-secret-key

    // ANCHOR: encrypt-default
    let password: &str = "nostr";
    let encrypted: EncryptedSecretKey = secret_key.encrypt(password)?;
    // ANCHOR_END: encrypt-default
    
    println!("Encrypted secret key: {}", encrypted.to_bech32()?);

    // ANCHOR: encrypt-custom
    let encrypted: EncryptedSecretKey = EncryptedSecretKey::new(&secret_key, password, 12, KeySecurity::Weak)?;
    // ANCHOR_END: encrypt-custom

    println!("Encrypted secret key (custom): {}", encrypted.to_bech32()?);

    Ok(())
}

pub fn decrypt() -> Result<()> {
    // ANCHOR: parse-ncryptsec
    let encrypted: EncryptedSecretKey = EncryptedSecretKey::from_bech32("ncryptsec1qgg9947rlpvqu76pj5ecreduf9jxhselq2nae2kghhvd5g7dgjtcxfqtd67p9m0w57lspw8gsq6yphnm8623nsl8xn9j4jdzz84zm3frztj3z7s35vpzmqf6ksu8r89qk5z2zxfmu5gv8th8wclt0h4p")?;
    // ANCHOR_END: parse-ncryptsec

    // ANCHOR: decrypt
    let secret_key: SecretKey = encrypted.to_secret_key("nostr")?;
    // ANCHOR_END: decrypt

    println!("Decrypted secret key: {}", secret_key.to_bech32()?);

    Ok(())
}
// ANCHOR_END: full
