// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::AddrParseError;

use tracing::subscriber::SetGlobalDefaultError;
use uniffi::Error;

pub type Result<T, E = NostrSdkError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum NostrSdkError {
    Generic { err: String },
}

impl std::error::Error for NostrSdkError {}

impl fmt::Display for NostrSdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic { err } => write!(f, "{err}"),
        }
    }
}

impl From<nostr_ffi::NostrError> for NostrSdkError {
    fn from(e: nostr_ffi::NostrError) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<SetGlobalDefaultError> for NostrSdkError {
    fn from(e: SetGlobalDefaultError) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr_sdk::client::Error> for NostrSdkError {
    fn from(e: nostr_sdk::client::Error) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr_sdk::relay::Error> for NostrSdkError {
    fn from(e: nostr_sdk::relay::Error) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<AddrParseError> for NostrSdkError {
    fn from(e: AddrParseError) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr_sdk::url::ParseError> for NostrSdkError {
    fn from(e: nostr_sdk::url::ParseError) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr_sdk::database::DatabaseError> for NostrSdkError {
    fn from(e: nostr_sdk::database::DatabaseError) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr_sdk::SQLiteError> for NostrSdkError {
    fn from(e: nostr_sdk::SQLiteError) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}
