// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC Options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::time::Duration;

use nostr_relay_pool::RelayOptions;

/// NWC options
#[derive(Debug, Clone)]
pub struct NostrWalletConnectOptions {
    pub(super) relay: RelayOptions,
    pub(super) timeout: Duration,
}

impl Default for NostrWalletConnectOptions {
    fn default() -> Self {
        Self {
            relay: RelayOptions::default(),
            timeout: Duration::from_secs(10),
        }
    }
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

    /// Set NWC requests timeout (default: 10 secs)
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
