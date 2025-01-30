// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;

use nostr::nips::nip65;
use uniffi::Enum;

use crate::protocol::event::Event;

#[derive(Enum)]
pub enum RelayMetadata {
    /// Read
    Read,
    /// Write
    Write,
}

impl From<RelayMetadata> for nip65::RelayMetadata {
    fn from(value: RelayMetadata) -> Self {
        match value {
            RelayMetadata::Read => Self::Read,
            RelayMetadata::Write => Self::Write,
        }
    }
}

impl From<nip65::RelayMetadata> for RelayMetadata {
    fn from(value: nip65::RelayMetadata) -> Self {
        match value {
            nip65::RelayMetadata::Read => Self::Read,
            nip65::RelayMetadata::Write => Self::Write,
        }
    }
}

/// Extracts the relay info (url, optional read/write flag) from the event
#[uniffi::export]
pub fn extract_relay_list(event: &Event) -> HashMap<String, Option<RelayMetadata>> {
    nip65::extract_relay_list(event.deref())
        .map(|(s, r)| (s.to_string(), r.map(|r| r.into())))
        .collect()
}
