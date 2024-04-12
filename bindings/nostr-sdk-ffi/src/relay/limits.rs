// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::helper::unwrap_or_clone_arc;
use nostr_sdk::pool::relay;
use uniffi::Object;

/// Relay Limits
#[derive(Debug, Clone, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
pub struct RelayLimits {
    inner: relay::RelayLimits,
}

impl Deref for RelayLimits {
    type Target = relay::RelayLimits;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl RelayLimits {
    /// Construct with default limits
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: relay::RelayLimits::default(),
        }
    }

    /// Disable all limits
    #[uniffi::constructor]
    pub fn disable() -> Self {
        Self {
            inner: relay::RelayLimits::disable(),
        }
    }

    /// Maximum size of normalised JSON, in bytes (default: 5_250_000)
    pub fn message_max_size(self: Arc<Self>, max_size: Option<u32>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner.messages.max_size = max_size;
        builder
    }

    /// Maximum size of normalised JSON, in bytes (default: 70_000)
    pub fn event_max_size(self: Arc<Self>, max_size: Option<u32>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner.events.max_size = max_size;
        builder
    }

    /// Maximum number of tags allowed (default: 2_000)
    pub fn event_max_num_tags(self: Arc<Self>, max_num_tags: Option<u16>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner.events.max_num_tags = max_num_tags;
        builder
    }
}
