// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::prelude::*;

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";

fn main() -> Result<()> {
    let secret_key = SecretKey::from_str(ALICE_SK)?;
    let alice_keys = Keys::new(secret_key);

    let public_key =
        PublicKey::from_bech32("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")?;
    let relays = [Url::parse("wss://relay.damus.io").unwrap()];
    let msg = "Zap!";
    let data = ZapRequestData::new(public_key, relays).message(msg);

    let public_zap: Event = EventBuilder::public_zap_request(data.clone()).to_event(&alice_keys)?;
    println!("Public zap request: {public_zap:#?}");

    let anon_zap: Event = nip57::anonymous_zap_request(data.clone())?;
    println!("Anonymous zap request: {anon_zap:#?}");

    let private_zap: Event = nip57::private_zap_request(data, &alice_keys)?;
    println!("Private zap request: {private_zap:#?}");

    Ok(())
}
