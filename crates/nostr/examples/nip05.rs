// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    let public_key =
        PublicKey::parse("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")?;

    let address = Nip05Address::parse("0xtr@oxtr.dev")?;

    println!("Url: {}", address.url());

    let json = r#"{
  "names": {
    "nostr-tool-test-user": "94a9eb13c37b3c1519169a426d383c51530f4fe8f693c62f32b321adfdd4ec7f",
    "robosatsob": "3b57518d02e6acfd5eb7198530b2e351e5a52278fb2499d14b66db2b5791c512",
    "0xtr": "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
    "_": "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a"
  },
  "relays": {
    "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a": [ "wss://nostr.oxtr.dev", "wss://relay.damus.io", "wss://relay.nostr.band" ]
  }
}"#;

    if nip05::verify_from_raw_json(&public_key, &address, json)? {
        println!("NIP05 verified");
    } else {
        println!("NIP05 NOT verified");
    }

    let profile: Nip05Profile = Nip05Profile::from_raw_json(&address, json)?;
    println!("Public key: {}", profile.public_key);
    println!("Relays: {:?}", profile.relays);
    println!("Relays (NIP46): {:?}", profile.nip46);

    Ok(())
}
