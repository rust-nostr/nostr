// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use nostr::prelude::*;
use nostr::serde_json;
use nostr_database::prelude::*;
use nostr_gossip::error::GossipError;
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
    /// Gossip error
    Gossip(GossipError),
    /// [`EventBuilder`] error
    EventBuilder(event::builder::Error),
    /// Json error
    Json(serde_json::Error),
    /// Shared state error
    SharedState(SharedStateError),
    /// NIP59
    #[cfg(feature = "nip59")]
    NIP59(nip59::Error),
    /// Broken down filters for gossip are empty
    GossipFiltersEmpty,
    /// Private message (NIP17) relays not found
    PrivateMsgRelaysNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Relay(e) => e.fmt(f),
            Self::RelayPool(e) => e.fmt(f),
            Self::Database(e) => e.fmt(f),
            Self::Signer(e) => e.fmt(f),
            Self::Gossip(e) => e.fmt(f),
            Self::EventBuilder(e) => e.fmt(f),
            Self::Json(e) => e.fmt(f),
            Self::SharedState(e) => e.fmt(f),
            #[cfg(feature = "nip59")]
            Self::NIP59(e) => e.fmt(f),
            Self::GossipFiltersEmpty => {
                f.write_str("gossip broken down filters are empty")
            }
            Self::PrivateMsgRelaysNotFound => f.write_str("Private message relays not found. The user is not ready to receive private messages."),
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

impl From<GossipError> for Error {
    fn from(e: GossipError) -> Self {
        Self::Gossip(e)
    }
}

impl From<event::builder::Error> for Error {
    fn from(e: event::builder::Error) -> Self {
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
