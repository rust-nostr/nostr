// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::key::FromSkStr;
use nostr::{EventBuilder, Keys, Result};

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";
const BOB_SK: &str = "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";

fn main() -> Result<()> {
    let alice_keys = Keys::from_sk_str(ALICE_SK)?;
    let bob_keys = Keys::from_sk_str(BOB_SK)?;

    let alice_encrypted_msg = EventBuilder::new_encrypted_direct_msg(
        &alice_keys,
        bob_keys.public_key(),
        "Hey bob this is alice",
        None,
    )?
    .to_event(&alice_keys)?;

    println!("{}", alice_encrypted_msg.as_json());

    Ok(())
}
