use nostr::prelude::*;

pub fn run() -> Result<()> {
    let keys = Keys::generate();

    let pk =
        PublicKey::from_hex("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")?;

    let ciphertext = nip44::encrypt(keys.secret_key(), &pk, "my message", nip44::Version::V2)?;
    println!("Encrypted: {ciphertext}");

    let plaintext = nip44::decrypt(keys.secret_key(), &pk, ciphertext)?;
    println!("Decrypted: {plaintext}");

    Ok(())
}
