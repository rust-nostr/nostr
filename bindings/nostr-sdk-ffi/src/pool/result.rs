// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::sync::Arc;

use nostr_ffi::EventId;
use nostr_sdk::{pool, SubscriptionId};
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

impl From<pool::Output<()>> for Output {
    fn from(output: pool::Output<()>) -> Self {
        Self {
            success: output.success.into_iter().map(|u| u.to_string()).collect(),
            failed: output
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

impl From<pool::Output<nostr_sdk::EventId>> for SendEventOutput {
    fn from(output: pool::Output<nostr_sdk::EventId>) -> Self {
        Self {
            id: Arc::new(output.val.into()),
            output: Output {
                success: output.success.into_iter().map(|u| u.to_string()).collect(),
                failed: output
                    .failed
                    .into_iter()
                    .map(|(u, e)| (u.to_string(), e))
                    .collect(),
            },
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

impl From<pool::Output<SubscriptionId>> for SubscribeOutput {
    fn from(output: pool::Output<SubscriptionId>) -> Self {
        Self {
            id: output.val.to_string(),
            output: Output {
                success: output.success.into_iter().map(|u| u.to_string()).collect(),
                failed: output
                    .failed
                    .into_iter()
                    .map(|(u, e)| (u.to_string(), e))
                    .collect(),
            },
        }
    }
}
