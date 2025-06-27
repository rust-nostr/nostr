// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC error

use std::fmt;

use nostr::nips::nip47;
use nostr_relay_pool::pool;

/// NWC error
#[derive(Debug)]
pub enum Error {
    /// NIP47 error
    NIP47(nip47::Error),
    /// Relay Pool
    Pool(pool::Error),
    /// Premature exit
    PrematureExit,
    /// Request timeout
    Timeout,
    /// Handler error
    Handler(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NIP47(e) => write!(f, "{e}"),
            Self::Pool(e) => write!(f, "{e}"),
            Self::PrematureExit => write!(f, "premature exit"),
            Self::Timeout => write!(f, "timeout"),
            Self::Handler(e) => write!(f, "handler error: {e}"),
        }
    }
}

impl From<nip47::Error> for Error {
    fn from(e: nip47::Error) -> Self {
        Self::NIP47(e)
    }
}

impl From<pool::Error> for Error {
    fn from(e: pool::Error) -> Self {
        Self::Pool(e)
    }
}
