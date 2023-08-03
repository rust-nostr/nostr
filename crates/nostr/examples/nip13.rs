// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use nostr::prelude::*;

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let secret_key = SecretKey::from_str(ALICE_SK)?;
    let alice_keys = Keys::new(secret_key);

    let difficulty = 20; // leading zero bits
    let msg_content = "This is a Nostr message with embedded proof-of-work";

    let builder = EventBuilder::new_text_note(msg_content, &[]);
    // or
    // let builder = EventBuilder::new(Kind::TextNote, msg_content, &[]);

    let event: Event = builder.to_pow_event(&alice_keys, difficulty)?;

    event.verify()?;

    println!("{:#?}", event);

    Ok(())
}
