// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;

fn main() -> Result<()> {
    let keys = Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")?;

    let difficulty = 20; // leading zero bits
    let msg_content = "This is a Nostr message with embedded proof-of-work";

    let event: Event = EventBuilder::text_note(msg_content)
        .pow(difficulty)
        .sign_with_keys(&keys)?;

    println!("{}", event.as_json());

    Ok(())
}
