// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;
use std::time::Duration;

use nostr_sdk::client::options;
use nostr_sdk::pool;
use uniffi::{Enum, Object};

use crate::error::Result;
use crate::relay::{ConnectionMode, RelayFilteringMode, RelayLimits};

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

    /// Automatically start connection with relays (default: false)
    ///
    /// When set to `true`, there isn't the need of calling the connect methods.
    pub fn autoconnect(&self, val: bool) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.autoconnect(val);
        builder
    }

    /// Minimum POW difficulty for received events
    pub fn min_pow(&self, difficulty: u8) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.min_pow(difficulty);
        builder
    }

    pub fn req_filters_chunk_size(&self, req_filters_chunk_size: u8) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.req_filters_chunk_size(req_filters_chunk_size);
        builder
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    pub fn automatic_authentication(&self, enabled: bool) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.automatic_authentication(enabled);
        builder
    }

    /// Enable gossip model (default: false)
    pub fn gossip(&self, enabled: bool) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.gossip(enabled);
        builder
    }

    /// Connection
    pub fn connection(&self, connection: &Connection) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.connection(connection.deref().clone());
        builder
    }

    /// Set custom relay limits
    pub fn relay_limits(&self, limits: &RelayLimits) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.relay_limits(limits.deref().clone());
        builder
    }

    /// Set max latency (default: None)
    ///
    /// Relays with an avg. latency greater that this value will be skipped.
    pub fn max_avg_latency(&self, max: Duration) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.max_avg_latency(max);
        builder
    }

    /// Set filtering mode (default: blacklist)
    pub fn filtering_mode(&self, mode: RelayFilteringMode) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.filtering_mode(mode.into());
        builder
    }
}

/// Connection target
#[derive(Enum)]
pub enum ConnectionTarget {
    /// Use proxy for all relays
    All,
    /// Use proxy only for `.onion` relays
    Onion,
}

impl From<ConnectionTarget> for options::ConnectionTarget {
    fn from(value: ConnectionTarget) -> Self {
        match value {
            ConnectionTarget::All => Self::All,
            ConnectionTarget::Onion => Self::Onion,
        }
    }
}

/// Connection
#[derive(Debug, Clone, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Connection {
    inner: options::Connection,
}

impl Deref for Connection {
    type Target = options::Connection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Connection {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: options::Connection::default(),
        }
    }

    /// Set connection mode (default: direct)
    pub fn mode(&self, mode: ConnectionMode) -> Result<Self> {
        let mode: pool::ConnectionMode = mode.try_into()?;
        let mut builder = self.clone();
        builder.inner = builder.inner.mode(mode);
        Ok(builder)
    }

    /// Set connection target (default: all)
    pub fn target(&self, target: ConnectionTarget) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.target(target.into());
        builder
    }

    /// Set proxy (ex. `127.0.0.1:9050`)
    pub fn addr(&self, addr: &str) -> Result<Self> {
        let mut builder = self.clone();
        let addr: SocketAddr = addr.parse()?;
        builder.inner = builder.inner.proxy(addr);
        Ok(builder)
    }
}
