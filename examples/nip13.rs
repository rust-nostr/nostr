// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::cmp::max;
use std::error::Error;
use std::str::FromStr;
use std::time::Instant;

use log::info;
use nostr::event::Tag;
use nostr::{util, Event, Keys};
use secp256k1::SecretKey;

const ALICE_SK: &str = "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e";

// Run with:
// RUST_LOG=info cargo run --example nip13 --
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let alice_keys = Keys::new(SecretKey::from_str(ALICE_SK)?)?;

    let pow_difficulty = 10; // leading zero bits
    let msg_content = "This is a Nostr message with embedded proof-of-work";

    let targeted_difficulty_str = pow_difficulty.to_string();

    // Loop: generate IDs until desired number of leading zeroes is reached
    let now = Instant::now();
    let mut iterations = 0;
    let mut found_valid_hash = false;

    while !found_valid_hash {
        iterations += 1;

        let nonce = iterations; // Different value per iteration
        let nonce_str = nonce.to_string();
        let t = Tag::new("nonce", &nonce_str, &targeted_difficulty_str);

        let temp_note = Event::new_textnote(msg_content, &alice_keys, &vec![t])
            .expect("Error when creating textnote");
        let id = temp_note.id;

        let leading_zeroes = util::nip13::get_leading_zero_bits(id);
        if leading_zeroes >= pow_difficulty {
            found_valid_hash = true;

            info!("Found matching hash: {}", id);
            info!(
                "Leading zero bits: {} (min. required: {})",
                leading_zeroes, pow_difficulty
            );
            let iter_string = format!("{}", iterations);
            let l = iter_string.len();
            let f = iter_string.chars().next().unwrap();
            info!(
                "{} iterations (about {}x10^{} hashes) in {} seconds. Avg rate {} hashes/second",
                iterations,
                f,
                l - 1,
                now.elapsed().as_secs(),
                iterations * 1000 / max(1, now.elapsed().as_millis())
            );
            info!(
                "Nostr event JSON is: {}",
                serde_json::to_string_pretty(&temp_note).unwrap()
            );
        }
    }

    Ok(())
}
