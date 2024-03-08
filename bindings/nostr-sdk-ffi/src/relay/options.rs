// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::helper::unwrap_or_clone_arc;
use uniffi::{Enum, Object};

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

    /// Timeout for sending event (default: 10 secs)
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
