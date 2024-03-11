// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::helper::unwrap_or_clone_arc;
use nostr_sdk::nwc;
use uniffi::Object;

use crate::error::Result;

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

    /// Set proxy
    pub fn proxy(self: Arc<Self>, proxy: Option<String>) -> Result<Self> {
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse()?),
            None => None,
        };
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.proxy(proxy);
        Ok(builder)
    }
}
