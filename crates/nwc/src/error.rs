// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC error

use nostr::nips::nip47;
use nostr_zapper::ZapperError;
use thiserror::Error;

/// NWC error
#[derive(Debug, Error)]
pub enum Error {
    /// Zapper error
    #[error(transparent)]
    Zapper(#[from] ZapperError),
    /// NIP47 error
    #[error(transparent)]
    NIP47(#[from] nip47::Error),
    /// Relay
    #[error("relay: {0}")]
    Relay(#[from] nostr_relay_pool::relay::Error),
    /// Premature exit from listener
    #[error("premature exit from listener")]
    PrematureExit,
    /// Request timeout
    #[error("timeout")]
    Timeout,
}

impl From<Error> for ZapperError {
    fn from(e: Error) -> Self {
        Self::backend(e)
    }
}
