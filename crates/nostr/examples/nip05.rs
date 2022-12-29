// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::util::nips::nip05;
use nostr::Result;

fn main() -> Result<()> {
    env_logger::init();

    let public_key = XOnlyPublicKey::from_str(
        "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
    )?;

    if nip05::verify(public_key, "0xtr@oxtr.dev", None).is_ok() {
        println!("NIP-05 verified");
    } else {
        println!("NIP-05 NOT verified");
    }

    Ok(())
}
