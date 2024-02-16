// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;

use nostr_relay_pool::{RelayOptions, RelayPoolOptions};

/// NWC options
#[derive(Debug, Clone, Default)]
pub struct NostrWalletConnectOptions {
    pub(super) relay: RelayOptions,
    pub(super) pool: RelayPoolOptions,
}

impl NostrWalletConnectOptions {
    /// New default NWC options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set proxy
    #[cfg(not(target_arch = "wasm32"))]
    pub fn proxy(self, proxy: Option<SocketAddr>) -> Self {
        Self {
            relay: self.relay.proxy(proxy),
            ..self
        }
    }

    /// Automatically shutdown relay pool on drop
    pub fn shutdown_on_drop(self, shutdown_on_drop: bool) -> Self {
        Self {
            pool: self.pool.shutdown_on_drop(shutdown_on_drop),
            ..self
        }
    }
}
