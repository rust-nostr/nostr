// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use nostr::nips::nip51;
use uniffi::Record;

use crate::{EventId, PublicKey};

#[derive(Record)]
pub struct MuteList {
    pub public_keys: Vec<Arc<PublicKey>>,
    pub hashtags: Vec<String>,
    pub event_ids: Vec<Arc<EventId>>,
    pub words: Vec<String>,
}

impl From<MuteList> for nip51::MuteList {
    fn from(value: MuteList) -> Self {
        Self {
            public_keys: value.public_keys.into_iter().map(|p| **p).collect(),
            hashtags: value.hashtags,
            event_ids: value.event_ids.into_iter().map(|e| **e).collect(),
            words: value.words,
        }
    }
}
