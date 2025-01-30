// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::pool::relay;
use uniffi::Object;

use crate::protocol::event::Kind;

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

    // TODO: change default value in doc automatically
    /// Maximum size of normalized JSON, in bytes (default: 5MB)
    pub fn message_max_size(&self, max_size: Option<u32>) -> Self {
        let mut builder = self.clone();
        builder.inner.messages.max_size = max_size;
        builder
    }

    /// Maximum size of normalised JSON, in bytes (default: 70_000)
    pub fn event_max_size(&self, max_size: Option<u32>) -> Self {
        let mut builder = self.clone();
        builder.inner.events.max_size = max_size;
        builder
    }

    /// Maximum size per kind of normalised JSON, in bytes.
    pub fn event_max_size_per_kind(&self, kind: &Kind, max_size: Option<u32>) -> Self {
        let mut builder = self.clone();
        builder.inner.events = builder.inner.events.set_max_size_per_kind(**kind, max_size);
        builder
    }

    /// Maximum number of tags allowed (default: 2_000)
    pub fn event_max_num_tags(&self, max_num_tags: Option<u16>) -> Self {
        let mut builder = self.clone();
        builder.inner.events.max_num_tags = max_num_tags;
        builder
    }

    /// Maximum number of tags allowed per kind
    pub fn event_max_num_tags_per_kind(&self, kind: &Kind, max_num_tags: Option<u16>) -> Self {
        let mut builder = self.clone();
        builder.inner.events = builder
            .inner
            .events
            .set_max_num_tags_per_kind(**kind, max_num_tags);
        builder
    }
}
