// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;

use nostr_relay_pool::RelayOptions;

/// NWC options
#[derive(Debug, Clone, Default)]
pub struct NostrWalletConnectOptions {
    pub(super) relay: RelayOptions,
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
        }
    }

    /// Automatically shutdown relay pool on drop
    #[deprecated(since = "0.29.0", note = "No longer needed")]
    pub fn shutdown_on_drop(self, _shutdown_on_drop: bool) -> Self {
        self
    }
}
