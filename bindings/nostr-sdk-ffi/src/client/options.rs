// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::helper::unwrap_or_clone_arc;
use nostr_sdk::client::options;
use uniffi::{Enum, Object};

use crate::error::Result;
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

    /// Proxy
    pub fn proxy(self: Arc<Self>, proxy: &Proxy) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.proxy(**proxy);
        builder
    }

    /// Set custom relay limits
    pub fn relay_limits(self: Arc<Self>, limits: &RelayLimits) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.relay_limits(**limits);
        builder
    }
}

/// Proxy target
#[derive(Enum)]
pub enum ProxyTarget {
    /// Use proxy for all relays
    All,
    /// Use proxy only for `.onion` relays
    Onion,
}

impl From<ProxyTarget> for options::ProxyTarget {
    fn from(value: ProxyTarget) -> Self {
        match value {
            ProxyTarget::All => Self::All,
            ProxyTarget::Onion => Self::Onion,
        }
    }
}

/// Proxy
#[derive(Clone, Object)]
pub struct Proxy {
    inner: options::Proxy,
}

impl Deref for Proxy {
    type Target = options::Proxy;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Proxy {
    /// Compose proxy (ex. `127.0.0.1:9050`)
    #[uniffi::constructor]
    pub fn new(addr: &str) -> Result<Self> {
        let addr: SocketAddr = addr.parse()?;
        Ok(Self {
            inner: options::Proxy::new(addr),
        })
    }

    /// Set proxy target (default: all)
    pub fn target(self: Arc<Self>, target: ProxyTarget) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.target(target.into());
        builder
    }
}
