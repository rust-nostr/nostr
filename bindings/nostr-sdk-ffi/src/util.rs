// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::RelayUrl;

use crate::error::Result;

pub fn parse_optional_relay_url(relay_url: Option<String>) -> Result<Option<RelayUrl>> {
    match relay_url {
        Some(url) => {
            if url.is_empty() {
                return Ok(None);
            }

            Ok(Some(RelayUrl::parse(&url)?))
        }
        None => Ok(None),
    }
}
