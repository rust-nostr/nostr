// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let public_key =
        PublicKey::parse("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")?;

    if nip05::verify(&public_key, "0xtr@oxtr.dev", None).await? {
        println!("NIP05 verified");
    } else {
        println!("NIP05 NOT verified");
    }

    let profile: Nip05Profile = nip05::profile("_@fiatjaf.com", None).await?;
    println!("Public key: {}", profile.public_key);
    println!("Relays: {:?}", profile.relays);
    println!("Relays (NIP46): {:?}", profile.nip46);

    Ok(())
}
