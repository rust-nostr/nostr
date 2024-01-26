// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::helper::unwrap_or_clone_arc;
use uniffi::{Enum, Object};

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
