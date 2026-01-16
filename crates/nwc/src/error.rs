// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC error

use std::fmt;

use nostr::nips::nip47;
use nostr_sdk::{client, relay};

/// NWC error
#[derive(Debug)]
pub enum Error {
    /// NIP47 error
    NIP47(nip47::Error),
    /// Client error
    Client(client::Error),
    /// Relay error
    Relay(relay::Error),
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
            Self::NIP47(e) => e.fmt(f),
            Self::Client(e) => e.fmt(f),
            Self::Relay(e) => e.fmt(f),
            Self::PrematureExit => f.write_str("premature exit"),
            Self::Timeout => f.write_str("timeout"),
            Self::Handler(e) => f.write_str(e),
        }
    }
}

impl From<nip47::Error> for Error {
    fn from(e: nip47::Error) -> Self {
        Self::NIP47(e)
    }
}

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        Self::Client(e)
    }
}

impl From<relay::Error> for Error {
    fn from(e: relay::Error) -> Self {
        Self::Relay(e)
    }
}
