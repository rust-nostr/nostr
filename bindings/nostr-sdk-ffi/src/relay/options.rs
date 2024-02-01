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
