// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC Options

use std::time::Duration;

use nostr_relay_pool::{ConnectionMode, RelayOptions};

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
            timeout: Duration::from_secs(60),
        }
    }
}

impl NostrWalletConnectOptions {
    /// New default NWC options
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set connection mode
    pub fn connection_mode(self, mode: ConnectionMode) -> Self {
        Self {
            relay: self.relay.connection_mode(mode),
            ..self
        }
    }

    /// Set NWC requests timeout (default: 10 secs)
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
