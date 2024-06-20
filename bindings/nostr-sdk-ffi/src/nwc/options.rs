// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::helper::unwrap_or_clone_arc;
use nostr_sdk::{nwc, pool};
use uniffi::Object;

use crate::error::Result;
use crate::relay::options::ConnectionMode;

/// NWC options
#[derive(Clone, Object)]
pub struct NostrWalletConnectOptions {
    inner: nwc::NostrWalletConnectOptions,
}

impl Deref for NostrWalletConnectOptions {
    type Target = nwc::NostrWalletConnectOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl NostrWalletConnectOptions {
    /// New default NWC options
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nwc::NostrWalletConnectOptions::new(),
        }
    }

    /// Set connection mode
    pub fn connection_mode(self: Arc<Self>, mode: ConnectionMode) -> Result<Self> {
        let mode: pool::ConnectionMode = mode.try_into()?;
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.connection_mode(mode);
        Ok(builder)
    }

    /// Set NWC requests timeout (default: 10 secs)
    pub fn timeout(self: Arc<Self>, timeout: Duration) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.timeout(timeout);
        builder
    }
}
