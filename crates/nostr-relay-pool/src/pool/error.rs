// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::convert::Infallible;
use std::fmt;

use nostr::types::url;
use nostr_database::DatabaseError;

use crate::__private::SharedStateError;
use crate::relay;

/// Relay Pool error
#[derive(Debug)]
pub enum Error {
    /// Shared state error
    SharedState(SharedStateError),
    /// Url parse error
    RelayUrl(url::Error),
    /// Relay error
    Relay(relay::Error),
    /// Database error
    Database(DatabaseError),
    /// Notification Handler error
    Handler(String),
    /// Too many relays
    TooManyRelays {
        /// Max numer allowed
        limit: usize,
    },
    /// No relays
    NoRelays,
    /// No relays specified
    NoRelaysSpecified,
    /// Negentropy reconciliation failed
    NegentropyReconciliationFailed,
    /// Relay not found
    RelayNotFound,
    /// Relay Pool is shutdown
    Shutdown,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SharedState(e) => e.fmt(f),
            Self::RelayUrl(e) => e.fmt(f),
            Self::Relay(e) => e.fmt(f),
            Self::Database(e) => e.fmt(f),
            Self::Handler(e) => e.fmt(f),
            Self::TooManyRelays { .. } => f.write_str("too many relays"),
            Self::NoRelays => f.write_str("no relays"),
            Self::NoRelaysSpecified => f.write_str("no relays specified"),
            Self::NegentropyReconciliationFailed => f.write_str("negentropy reconciliation failed"),
            Self::RelayNotFound => f.write_str("relay not found"),
            Self::Shutdown => f.write_str("relay pool is shutdown"),
        }
    }
}

impl From<SharedStateError> for Error {
    fn from(e: SharedStateError) -> Self {
        Self::SharedState(e)
    }
}

impl From<url::Error> for Error {
    fn from(e: url::Error) -> Self {
        Self::RelayUrl(e)
    }
}

impl From<relay::Error> for Error {
    fn from(e: relay::Error) -> Self {
        Self::Relay(e)
    }
}

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Self::Database(e)
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}
