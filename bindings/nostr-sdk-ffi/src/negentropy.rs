// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use nostr_ffi::{EventId, Timestamp};
use uniffi::Record;

#[derive(Record)]
pub struct NegentropyItem {
    pub id: Arc<EventId>,
    pub timestamp: Arc<Timestamp>,
}
