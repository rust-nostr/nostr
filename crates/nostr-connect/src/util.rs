// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::{nip04, nip44};
use nostr::{Event, SecretKey};

use crate::error::Error;

/// Decrypt a NIP46 message. Support both NIP04 and NIP44.
pub fn decrypt(secret_key: &SecretKey, event: &Event) -> Result<String, Error> {
    if event.content.contains("?iv=") {
        Ok(nip04::decrypt(
            secret_key,
            &event.pubkey,
            event.content.as_str(),
        )?)
    } else {
        Ok(nip44::decrypt(
            secret_key,
            &event.pubkey,
            event.content.as_str(),
        )?)
    }
}
