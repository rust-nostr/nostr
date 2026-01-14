// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay builder error

use std::{fmt, io};

#[cfg(feature = "tor")]
use async_wsocket::native::tor;
use nostr_sdk::pool;

/// Relay builder error
#[derive(Debug)]
pub enum Error {
    /// I/O error
    IO(io::Error),
    /// Tor error
    #[cfg(feature = "tor")]
    Tor(tor::Error),
    /// Relay pool error
    RelayPool(pool::Error),
    /// Relay already running
    AlreadyRunning,
    /// Premature exit
    PrematureExit,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "{e}"),
            #[cfg(feature = "tor")]
            Self::Tor(e) => write!(f, "{e}"),
            Self::RelayPool(e) => e.fmt(f),
            Self::AlreadyRunning => write!(f, "the relay is already running"),
            Self::PrematureExit => write!(f, "premature exit"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

#[cfg(feature = "tor")]
impl From<tor::Error> for Error {
    fn from(e: tor::Error) -> Self {
        Self::Tor(e)
    }
}

impl From<pool::Error> for Error {
    fn from(e: pool::Error) -> Self {
        Self::RelayPool(e)
    }
}
