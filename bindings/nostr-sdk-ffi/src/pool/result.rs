// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::sync::Arc;

use nostr_ffi::EventId;
use nostr_sdk::pool;
use uniffi::Record;

/// Output
///
/// Send or negentropy reconciliation output
#[derive(Record)]
pub struct Output {
    /// Set of relays that success
    pub success: Vec<String>,
    /// Map of relays that failed, with related errors.
    pub failed: HashMap<String, Option<String>>,
}

impl From<pool::Output> for Output {
    fn from(value: pool::Output) -> Self {
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
    /// Output
    pub output: Output,
}

impl From<pool::SendEventOutput> for SendEventOutput {
    fn from(value: pool::SendEventOutput) -> Self {
        Self {
            id: Arc::new(value.id.into()),
            output: value.output.into(),
        }
    }
}

/// Subscribe output
#[derive(Record)]
pub struct SubscribeOutput {
    /// Subscription ID
    pub id: String,
    /// Output
    pub output: Output,
}

impl From<pool::SubscribeOutput> for SubscribeOutput {
    fn from(value: pool::SubscribeOutput) -> Self {
        Self {
            id: value.id.to_string(),
            output: value.output.into(),
        }
    }
}
