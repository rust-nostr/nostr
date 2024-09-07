// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::net::AddrParseError;

use tracing::subscriber::SetGlobalDefaultError;
use uniffi::Error;

pub type Result<T, E = NostrSdkError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[uniffi(flat_error)]
pub enum NostrSdkError {
    Generic(String),
}

impl std::error::Error for NostrSdkError {}

impl fmt::Display for NostrSdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(e) => write!(f, "{e}"),
        }
    }
}

impl From<nostr_ffi::NostrError> for NostrSdkError {
    fn from(e: nostr_ffi::NostrError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<SetGlobalDefaultError> for NostrSdkError {
    fn from(e: SetGlobalDefaultError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::client::Error> for NostrSdkError {
    fn from(e: nostr_sdk::client::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::pool::relay::Error> for NostrSdkError {
    fn from(e: nostr_sdk::pool::relay::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::pool::pool::Error> for NostrSdkError {
    fn from(e: nostr_sdk::pool::pool::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<AddrParseError> for NostrSdkError {
    fn from(e: AddrParseError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::types::url::ParseError> for NostrSdkError {
    fn from(e: nostr_sdk::types::url::ParseError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::database::DatabaseError> for NostrSdkError {
    fn from(e: nostr_sdk::database::DatabaseError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::signer::Error> for NostrSdkError {
    fn from(e: nostr_sdk::signer::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::signer::nip46::Error> for NostrSdkError {
    fn from(e: nostr_sdk::signer::nip46::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::nwc::Error> for NostrSdkError {
    fn from(e: nostr_sdk::nwc::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<async_utility::thread::Error> for NostrSdkError {
    fn from(e: async_utility::thread::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_relay_builder::mock::Error> for NostrSdkError {
    fn from(e: nostr_relay_builder::mock::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}
