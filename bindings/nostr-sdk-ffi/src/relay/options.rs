// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::helper::unwrap_or_clone_arc;
use uniffi::{Enum, Object};

use super::RelayLimits;
use crate::error::Result;

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

    /// Set proxy
    pub fn proxy(self: Arc<Self>, proxy: Option<String>) -> Result<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse()?),
            None => None,
        };
        builder.inner = builder.inner.proxy(proxy);
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
    pub fn pow(self: Arc<Self>, diffculty: u8) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.pow(diffculty);
        builder
    }

    /// Update `pow` option
    pub fn update_pow_difficulty(&self, diffculty: u8) {
        self.inner.update_pow_difficulty(diffculty);
    }

    /// Enable/disable auto reconnection (default: true)
    pub fn reconnect(self: Arc<Self>, reconnect: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.reconnect(reconnect);
        builder
    }

    /// Update `reconnect` option
    pub fn update_reconnect(&self, reconnect: bool) {
        self.inner.update_reconnect(reconnect);
    }

    /// Retry connection time (default: 10 sec)
    ///
    /// Are allowed values `>=` 5 secs
    pub fn retry_sec(self: Arc<Self>, retry_sec: u64) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.retry_sec(retry_sec);
        builder
    }

    /// Set retry_sec option
    pub fn update_retry_sec(&self, retry_sec: u64) {
        self.inner.update_retry_sec(retry_sec);
    }

    /// Automatically adjust retry seconds based on success/attempts (default: true)
    pub fn adjust_retry_sec(self: Arc<Self>, adjust_retry_sec: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.adjust_retry_sec(adjust_retry_sec);
        builder
    }

    /// Set adjust_retry_sec option
    pub fn update_adjust_retry_sec(&self, adjust_retry_sec: bool) {
        self.inner.update_adjust_retry_sec(adjust_retry_sec);
    }

    /// Set custom limits
    pub fn limits(self: Arc<Self>, limits: &RelayLimits) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.limits(**limits);
        builder
    }
}

#[derive(Clone, Object)]
pub struct RelaySendOptions {
    inner: nostr_sdk::RelaySendOptions,
}

impl Deref for RelaySendOptions {
    type Target = nostr_sdk::RelaySendOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl RelaySendOptions {
    /// New default `RelaySendOptions`
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr_sdk::RelaySendOptions::default(),
        }
    }

    /// Skip wait for disconnected relay (default: true)
    pub fn skip_disconnected(self: Arc<Self>, value: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.skip_disconnected(value);
        builder
    }

    /// Skip wait for confirmation that message is sent (default: false)
    pub fn skip_send_confirmation(self: Arc<Self>, value: bool) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.skip_send_confirmation(value);
        builder
    }

    /// Timeout for sending event (default: 20 secs)
    ///
    /// If `None`, the default timeout will be used
    pub fn timeout(self: Arc<Self>, timeout: Option<Duration>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.timeout(timeout);
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

    /// Set [RelaySendOptions]
    pub fn send_opts(self: Arc<Self>, opts: &RelaySendOptions) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.send_opts(**opts);
        builder
    }
}

#[derive(Enum)]
pub enum NegentropyDirection {
    Up,
    Down,
    Both,
}

impl From<NegentropyDirection> for nostr_sdk::NegentropyDirection {
    fn from(value: NegentropyDirection) -> Self {
        match value {
            NegentropyDirection::Up => Self::Up,
            NegentropyDirection::Down => Self::Down,
            NegentropyDirection::Both => Self::Both,
        }
    }
}

#[derive(Clone, Object)]
pub struct NegentropyOptions {
    inner: nostr_sdk::NegentropyOptions,
}

impl Deref for NegentropyOptions {
    type Target = nostr_sdk::NegentropyOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl NegentropyOptions {
    /// New default options
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr_sdk::NegentropyOptions::new(),
        }
    }

    /// Timeout to check if negentropy it's supported (default: 10 secs)
    pub fn initial_timeout(self: Arc<Self>, timeout: Duration) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.initial_timeout(timeout);
        builder
    }

    /// Negentropy Sync direction (default: down)
    pub fn direction(self: Arc<Self>, direction: NegentropyDirection) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.direction(direction.into());
        builder
    }
}
