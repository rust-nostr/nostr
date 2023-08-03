// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::nips::nip05;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let public_key = XOnlyPublicKey::from_str(
        "b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a",
    )?;

    if nip05::verify_blocking(public_key, "0xtr@oxtr.dev", None).is_ok() {
        println!("NIP-05 verified");
    } else {
        println!("NIP-05 NOT verified");
    }

    let profile = nip05::get_profile_blocking("_@fiatjaf.com", None)?;
    println!("Profile example (including relays): {profile:#?}");

    Ok(())
}
