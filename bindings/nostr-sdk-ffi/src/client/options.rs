// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::helper::unwrap_or_clone_arc;
use uniffi::Object;

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

    pub fn wait_for_connection(self: Arc<Self>, wait: bool) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.wait_for_connection(wait);
        Arc::new(builder)
    }

    pub fn wait_for_send(self: Arc<Self>, wait: bool) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.wait_for_send(wait);
        Arc::new(builder)
    }

    pub fn wait_for_subscription(self: Arc<Self>, wait: bool) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.wait_for_subscription(wait);
        Arc::new(builder)
    }

    pub fn difficulty(self: Arc<Self>, difficulty: u8) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.difficulty(difficulty);
        Arc::new(builder)
    }

    pub fn req_filters_chunk_size(self: Arc<Self>, req_filters_chunk_size: u8) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.req_filters_chunk_size(req_filters_chunk_size);
        Arc::new(builder)
    }

    pub fn skip_disconnected_relays(self: Arc<Self>, skip: bool) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.skip_disconnected_relays(skip);
        Arc::new(builder)
    }

    pub fn timeout(self: Arc<Self>, timeout: Duration) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.timeout(timeout);
        Arc::new(builder)
    }

    pub fn send_timeout(self: Arc<Self>, send_timeout: Option<Duration>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.send_timeout(send_timeout);
        Arc::new(builder)
    }

    pub fn nip46_timeout(self: Arc<Self>, nip46_timeout: Option<Duration>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.nip46_timeout(nip46_timeout);
        Arc::new(builder)
    }
}
