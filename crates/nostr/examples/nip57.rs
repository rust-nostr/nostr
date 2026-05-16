// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    let keys = Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")?;

    let public_key =
        PublicKey::from_bech32("npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy")?;
    let relays = [RelayUrl::parse("wss://relay.damus.io")?];
    let data = ZapRequestData::new(public_key, relays).message("Zap!");

    let public_zap: Event = EventBuilder::public_zap_request(data.clone()).sign(&keys)?;
    println!("Public zap request: {}", public_zap.as_json());

    Ok(())
}
