// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use nostr::prelude::*;
use nostr::serde_json;
use nostr_database::prelude::*;
use nostr_relay_pool::__private::SharedStateError;
use nostr_relay_pool::prelude::*;

/// Client error
#[derive(Debug)]
pub enum Error {
    /// Relay error
    Relay(nostr_relay_pool::relay::Error),
    /// Relay Pool error
    RelayPool(pool::Error),
    /// Database error
    Database(DatabaseError),
    /// Signer error
    Signer(SignerError),
    /// [`EventBuilder`] error
    EventBuilder(builder::Error),
    /// Json error
    Json(serde_json::Error),
    /// Shared state error
    SharedState(SharedStateError),
    /// NIP59
    #[cfg(feature = "nip59")]
    NIP59(nip59::Error),
    /// Event not found
    EventNotFound(EventId),
    /// Impossible to zap
    ImpossibleToZap(String),
    /// Broken down filters for gossip are empty
    GossipFiltersEmpty,
    /// DMs relays not found
    DMsRelaysNotFound,
    /// Metadata not found
    MetadataNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Relay(e) => write!(f, "{e}"),
            Self::RelayPool(e) => write!(f, "{e}"),
            Self::Database(e) => write!(f, "{e}"),
            Self::Signer(e) => write!(f, "{e}"),
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::SharedState(e) => write!(f, "{e}"),
            #[cfg(feature = "nip59")]
            Self::NIP59(e) => write!(f, "{e}"),
            Self::EventNotFound(id) => {
                write!(f, "event not found: {id}")
            }
            Self::ImpossibleToZap(id) => {
                write!(f, "impossible to send zap: {id}")
            }
            Self::GossipFiltersEmpty => {
                write!(f, "gossip broken down filters are empty")
            }
            Self::DMsRelaysNotFound => write!(f, "DMs relays not found"),
            Self::MetadataNotFound => write!(f, "metadata not found"),
        }
    }
}

impl From<nostr_relay_pool::relay::Error> for Error {
    fn from(e: nostr_relay_pool::relay::Error) -> Self {
        Self::Relay(e)
    }
}

impl From<pool::Error> for Error {
    fn from(e: pool::Error) -> Self {
        Self::RelayPool(e)
    }
}

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Self::Database(e)
    }
}

impl From<SignerError> for Error {
    fn from(e: SignerError) -> Self {
        Self::Signer(e)
    }
}

impl From<builder::Error> for Error {
    fn from(e: builder::Error) -> Self {
        Self::EventBuilder(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<SharedStateError> for Error {
    fn from(e: SharedStateError) -> Self {
        Self::SharedState(e)
    }
}

#[cfg(feature = "nip59")]
impl From<nip59::Error> for Error {
    fn from(e: nip59::Error) -> Self {
        Self::NIP59(e)
    }
}
