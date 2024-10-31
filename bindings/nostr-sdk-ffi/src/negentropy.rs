// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use uniffi::Record;

use crate::protocol::{EventId, Timestamp};

#[derive(Record)]
pub struct NegentropyItem {
    pub id: Arc<EventId>,
    pub timestamp: Arc<Timestamp>,
}
