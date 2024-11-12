// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use nostr_sdk::pool;
use uniffi::{Enum, Object};

use super::{RelayFilteringMode, RelayLimits};
use crate::error::{NostrSdkError, Result};
use crate::protocol::helper::unwrap_or_clone_arc;

#[derive(Enum)]
pub enum ConnectionMode {
    Direct,
    Proxy { addr: String },
    Tor { custom_path: Option<String> },
}

impl From<pool::ConnectionMode> for ConnectionMode {
    fn from(mode: pool::ConnectionMode) -> Self {
        match mode {
            pool::ConnectionMode::Direct => Self::Direct,
            pool::ConnectionMode::Proxy(addr) => Self::Proxy {
                addr: addr.to_string(),
            },
            pool::ConnectionMode::Tor { custom_path } => Self::Tor {
                custom_path: custom_path.map(|p| p.to_string_lossy().into_owned()),
            },
        }
    }
}

impl TryFrom<ConnectionMode> for pool::ConnectionMode {
    type Error = NostrSdkError;

    fn try_from(mode: ConnectionMode) -> Result<Self, Self::Error> {
        match mode {
            ConnectionMode::Direct => Ok(Self::Direct),
            ConnectionMode::Proxy { addr } => Ok(Self::Proxy(addr.parse()?)),
            ConnectionMode::Tor { custom_path } => Ok(Self::Tor {
                custom_path: custom_path.map(PathBuf::from),
            }),
        }
    }
}

/// `Relay` options
#[derive(Clone, Object)]
pub struct RelayOptions {
    inner: nostr_sdk::RelayOptions,
}

impl Deref for RelayOptions {
    type Target = nostr_sdk::RelayOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr_sdk::RelayOptions> for RelayOptions {
    fn from(inner: nostr_sdk::RelayOptions) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl RelayOptions {
    /// New default relay options
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr_sdk::RelayOptions::new(),
        }
    }

    /// Set connection mode
    pub fn connection_mode(self: Arc<Self>, mode: ConnectionMode) -> Result<Self> {
        let mode: pool::ConnectionMode = mode.try_into()?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.connection_mode(mode);
        Ok(builder)
    }

    /*/// Set Relay Service Flags
    pub fn flags(mut self, flags: RelayServiceFlags) -> Self {
        self.flags = AtomicRelayServiceFlags::new(flags);
        self
    }*/

    /// Set read flag
    pub fn read(self: Arc<Self>, read: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.read(read);
        builder
    }

    /// Set write flag
    pub fn write(self: Arc<Self>, write: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.write(write);
        builder
    }

    /// Set ping flag
    pub fn ping(self: Arc<Self>, ping: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.ping(ping);
        builder
    }

    /// Minimum POW for received events (default: 0)
    pub fn pow(self: Arc<Self>, difficulty: u8) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.pow(difficulty);
        builder
    }

    /// Update `pow` option
    pub fn update_pow_difficulty(&self, difficulty: u8) {
        self.inner.update_pow_difficulty(difficulty);
    }

    /// Enable/disable auto reconnection (default: true)
    pub fn reconnect(self: Arc<Self>, reconnect: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.reconnect(reconnect);
        builder
    }

    /// Retry interval (default: 10 sec)
    ///
    /// Minimum allowed value is `5 secs`
    pub fn retry_interval(self: Arc<Self>, interval: Duration) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.retry_interval(interval);
        builder
    }

    /// Automatically adjust retry interval based on success/attempts (default: true)
    pub fn adjust_retry_interval(self: Arc<Self>, adjust_retry_interval: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.adjust_retry_interval(adjust_retry_interval);
        builder
    }

    /// Set custom limits
    pub fn limits(self: Arc<Self>, limits: &RelayLimits) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.limits(limits.deref().clone());
        builder
    }

    /// Set max latency (default: None)
    ///
    /// Relay with an avg. latency greater that this value will be skipped.
    pub fn max_avg_latency(self: Arc<Self>, max: Option<Duration>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.max_avg_latency(max);
        builder
    }

    /// Set filtering mode (default: blacklist)
    pub fn filtering_mode(self: Arc<Self>, mode: RelayFilteringMode) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.filtering_mode(mode.into());
        builder
    }
}

/// Filter options
#[derive(Enum)]
pub enum FilterOptions {
    /// Exit on EOSE
    ExitOnEOSE,
    /// After EOSE is received, keep listening for N more events that match the filter, then return
    WaitForEventsAfterEOSE { num: u16 },
    /// After EOSE is received, keep listening for matching events for `Duration` more time, then return
    WaitDurationAfterEOSE { duration: Duration },
}

impl From<FilterOptions> for nostr_sdk::FilterOptions {
    fn from(value: FilterOptions) -> Self {
        match value {
            FilterOptions::ExitOnEOSE => Self::ExitOnEOSE,
            FilterOptions::WaitForEventsAfterEOSE { num } => Self::WaitForEventsAfterEOSE(num),
            FilterOptions::WaitDurationAfterEOSE { duration } => {
                Self::WaitDurationAfterEOSE(duration)
            }
        }
    }
}

/// Auto-closing subscribe options
#[derive(Clone, Object)]
pub struct SubscribeAutoCloseOptions {
    inner: nostr_sdk::SubscribeAutoCloseOptions,
}

impl Deref for SubscribeAutoCloseOptions {
    type Target = nostr_sdk::SubscribeAutoCloseOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl SubscribeAutoCloseOptions {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr_sdk::SubscribeAutoCloseOptions::default(),
        }
    }

    /// Close subscription when `FilterOptions` is satisfied
    pub fn filter(self: Arc<Self>, filter: FilterOptions) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.filter(filter.into());
        builder
    }

    /// Automatically close subscription after `Duration`
    pub fn timeout(self: Arc<Self>, timeout: Option<Duration>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.timeout(timeout);
        builder
    }
}

/// Subscribe options
#[derive(Clone, Object)]
pub struct SubscribeOptions {
    inner: nostr_sdk::SubscribeOptions,
}

impl Deref for SubscribeOptions {
    type Target = nostr_sdk::SubscribeOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl SubscribeOptions {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr_sdk::SubscribeOptions::default(),
        }
    }

    /// Set auto-close conditions
    pub fn close_on(self: Arc<Self>, opts: Option<Arc<SubscribeAutoCloseOptions>>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.close_on(opts.map(|o| **o));
        builder
    }
}

#[derive(Enum)]
pub enum SyncDirection {
    Up,
    Down,
    Both,
}

impl From<SyncDirection> for nostr_sdk::SyncDirection {
    fn from(value: SyncDirection) -> Self {
        match value {
            SyncDirection::Up => Self::Up,
            SyncDirection::Down => Self::Down,
            SyncDirection::Both => Self::Both,
        }
    }
}

#[derive(Clone, Object)]
pub struct SyncOptions {
    inner: nostr_sdk::SyncOptions,
}

impl Deref for SyncOptions {
    type Target = nostr_sdk::SyncOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl SyncOptions {
    /// New default options
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr_sdk::SyncOptions::new(),
        }
    }

    /// Timeout to check if negentropy it's supported (default: 10 secs)
    pub fn initial_timeout(self: Arc<Self>, timeout: Duration) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.initial_timeout(timeout);
        builder
    }

    /// Sync Sync direction (default: down)
    pub fn direction(self: Arc<Self>, direction: SyncDirection) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.direction(direction.into());
        builder
    }

    /// Dry run
    ///
    /// Just check what event are missing: execute reconciliation but WITHOUT
    /// getting/sending full events.
    pub fn dry_run(self: Arc<Self>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.dry_run();
        builder
    }
}
