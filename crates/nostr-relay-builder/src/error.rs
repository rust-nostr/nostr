// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay builder error

use std::{fmt, io};

#[cfg(feature = "tor")]
use async_wsocket::native::tor;

/// Relay builder error
#[derive(Debug)]
pub enum Error {
    /// I/O error
    IO(io::Error),
    /// Tor error
    #[cfg(feature = "tor")]
    Tor(tor::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "{e}"),
            #[cfg(feature = "tor")]
            Self::Tor(e) => write!(f, "{e}"),
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
