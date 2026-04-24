// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::num::NonZeroU8;

use nostr::prelude::*;

fn main() -> Result<()> {
    let keys = Keys::parse("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")?;

    let difficulty = NonZeroU8::new(20).unwrap(); // leading zero bits
    let msg_content = "This is a Nostr message with embedded proof-of-work";

    // Build unsigned event
    let unsigned: UnsignedEvent = EventBuilder::text_note(msg_content).build(keys.public_key);

    // Compute POW
    #[cfg(not(feature = "pow-multi-thread"))]
    let unsigned = unsigned.mine(SingleThreadPow, difficulty)?;
    #[cfg(feature = "pow-multi-thread")]
    let unsigned = unsigned.mine(MultiThreadPow, difficulty)?;

    // Sign event
    let event: Event = unsigned.sign_with_keys(&keys)?;

    println!("{}", event.as_json());

    Ok(())
}
