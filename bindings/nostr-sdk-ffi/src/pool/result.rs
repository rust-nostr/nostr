// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::sync::Arc;

use nostr_ffi::EventId;
use nostr_sdk::pool;
use uniffi::Record;

/// Send output
#[derive(Record)]
pub struct SendOutput {
    /// Set of relay urls to which the message/s was successfully sent
    pub success: Vec<String>,
    /// Map of relay urls with related errors where the message/s wasn't sent
    pub failed: HashMap<String, Option<String>>,
}

impl From<pool::SendOutput> for SendOutput {
    fn from(value: pool::SendOutput) -> Self {
        Self {
            success: value.success.into_iter().map(|u| u.to_string()).collect(),
            failed: value
                .failed
                .into_iter()
                .map(|(u, e)| (u.to_string(), e))
                .collect(),
        }
    }
}

/// Send event output
#[derive(Record)]
pub struct SendEventOutput {
    /// Event ID
    pub id: Arc<EventId>,
    /// Set of relay urls to which the message/s was successfully sent
    pub success: Vec<String>,
    /// Map of relay urls with related errors where the message/s wasn't sent
    pub failed: HashMap<String, Option<String>>,
}

impl From<pool::SendEventOutput> for SendEventOutput {
    fn from(value: pool::SendEventOutput) -> Self {
        Self {
            id: Arc::new(value.id.into()),
            success: value.success.into_iter().map(|u| u.to_string()).collect(),
            failed: value
                .failed
                .into_iter()
                .map(|(u, e)| (u.to_string(), e))
                .collect(),
        }
    }
}

/// Negentropy reconciliation output
#[derive(Record)]
pub struct ReconciliationOutput {
    /// Set of relay urls to which the negentropy reconciliation success
    pub success: Vec<String>,
    /// Map of relay urls with related errors where the negentropy reconciliation failed
    pub failed: HashMap<String, Option<String>>,
}

impl From<pool::ReconciliationOutput> for ReconciliationOutput {
    fn from(value: pool::ReconciliationOutput) -> Self {
        Self {
            success: value.success.into_iter().map(|u| u.to_string()).collect(),
            failed: value
                .failed
                .into_iter()
                .map(|(u, e)| (u.to_string(), e))
                .collect(),
        }
    }
}
