// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay builder error

use std::{fmt, io};

use nostr_database::DatabaseError;
use nostr_sdk::client;

/// Relay builder error
#[derive(Debug)]
pub enum Error {
    /// I/O error
    IO(io::Error),
    /// Database error
    Database(DatabaseError),
    /// Client error
    Client(client::Error),
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
