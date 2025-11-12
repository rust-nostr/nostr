// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Error

use std::fmt;

use sqlx::migrate::MigrateError;

/// Nostr SQL error
#[derive(Debug)]
pub enum Error {
    /// SQLx error
    Sqlx(sqlx::Error),
    /// Migration error
    Migrate(MigrateError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlx(e) => write!(f, "{e}"),
            Self::Migrate(e) => write!(f, "{e}"),
        }
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
