// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay builder error

use std::{fmt, io};

use nostr::Event;
use nostr_database::DatabaseError;
use nostr_sdk::client;
use tokio::sync::broadcast;

/// Relay builder error
#[derive(Debug)]
pub enum Error {
    /// I/O error
    IO(io::Error),
    /// Database error
    Database(DatabaseError),
    /// Client error
    Client(client::Error),
    /// Nostr protocol error
    Protocol(nostr::error::Error),
    /// Other error
    Other(String),
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
            Self::Database(e) => write!(f, "{e}"),
            Self::Client(e) => e.fmt(f),
            Self::Protocol(e) => e.fmt(f),
            Self::Other(e) => f.write_str(e),
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

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Self::Database(e)
    }
}

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        Self::Client(e)
    }
}

impl From<nostr::error::Error> for Error {
    fn from(e: nostr::error::Error) -> Self {
        Self::Protocol(e)
    }
}

impl From<async_wsocket::Error> for Error {
    fn from(e: async_wsocket::Error) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<tokio::sync::TryAcquireError> for Error {
    fn from(e: tokio::sync::TryAcquireError) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<broadcast::error::SendError<Event>> for Error {
    fn from(e: broadcast::error::SendError<Event>) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<negentropy::Error> for Error {
    fn from(e: negentropy::Error) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<faster_hex::Error> for Error {
    fn from(e: faster_hex::Error) -> Self {
        Self::Other(e.to_string())
    }
}
