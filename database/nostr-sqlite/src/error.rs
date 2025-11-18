//! Error

use std::fmt;
use std::num::TryFromIntError;

use nostr::{event, key, secp256k1};
use sqlx::migrate::MigrateError;

/// Nostr SQL error
#[derive(Debug)]
pub enum Error {
    /// TryFromInt error
    TryFromInt(TryFromIntError),
    /// SQLx error
    Sqlx(sqlx::Error),
    /// Migration error
    Migrate(MigrateError),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Event error
    Event(event::Error),
    /// Key error
    Key(key::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TryFromInt(e) => write!(f, "{e}"),
            Self::Sqlx(e) => write!(f, "{e}"),
            Self::Migrate(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::Key(e) => write!(f, "{e}"),
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Self::TryFromInt(e)
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Self::Sqlx(e)
    }
}

impl From<MigrateError> for Error {
    fn from(e: MigrateError) -> Self {
        Self::Migrate(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
    }
}
