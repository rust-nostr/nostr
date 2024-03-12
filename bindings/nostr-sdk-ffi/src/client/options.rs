// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::helper::unwrap_or_clone_arc;
use uniffi::Object;

use crate::relay::RelayLimits;

#[derive(Clone, Object)]
pub struct Options {
    inner: nostr_sdk::Options,
}

impl Deref for Options {
    type Target = nostr_sdk::Options;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr_sdk::Options> for Options {
    fn from(inner: nostr_sdk::Options) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Options {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr_sdk::Options::new(),
        }
    }

    pub fn wait_for_send(self: Arc<Self>, wait: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.wait_for_send(wait);
        builder
    }

    pub fn wait_for_subscription(self: Arc<Self>, wait: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.wait_for_subscription(wait);
        builder
    }

    pub fn difficulty(self: Arc<Self>, difficulty: u8) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.difficulty(difficulty);
        builder
    }

    /// Minimum POW difficulty for received events
    pub fn min_pow(self: Arc<Self>, difficulty: u8) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.min_pow(difficulty);
        builder
    }

    pub fn req_filters_chunk_size(self: Arc<Self>, req_filters_chunk_size: u8) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.req_filters_chunk_size(req_filters_chunk_size);
        builder
    }

    pub fn skip_disconnected_relays(self: Arc<Self>, skip: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.skip_disconnected_relays(skip);
        builder
    }

    pub fn timeout(self: Arc<Self>, timeout: Duration) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.timeout(timeout);
        builder
    }

    /// Connection timeout (default: None)
    ///
    /// If set to `None`, the client will try to connect to the relays without waiting.
    pub fn connection_timeout(self: Arc<Self>, timeout: Option<Duration>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.connection_timeout(timeout);
        builder
    }

    pub fn send_timeout(self: Arc<Self>, send_timeout: Option<Duration>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.send_timeout(send_timeout);
        builder
    }

    /// Set custom relay limits
    pub fn relay_limits(self: Arc<Self>, limits: &RelayLimits) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.relay_limits(**limits);
        builder
    }
}
