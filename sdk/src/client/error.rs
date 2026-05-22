// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use nostr::serde_json;
use nostr_gossip::error::GossipError;

use crate::{pool, relay};

/// Client error
#[derive(Debug)]
pub enum Error {
    /// Nostr protocol error
    Protocol(nostr::error::Error),
    /// Relay error
    Relay(relay::Error),
    /// Relay Pool error
    RelayPool(pool::Error),
    /// Database error
    Database(nostr_database::error::Error),
    /// Gossip error
    Gossip(GossipError),
    /// Json error
    Json(serde_json::Error),
    /// Signer not configured
    SignerNotConfigured,
    /// Gossip is not configured
    GossipNotConfigured,
    /// Broken down filters for gossip are empty
    GossipFiltersEmpty,
    /// Private message (NIP17) relays not found
    PrivateMsgRelaysNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::Relay(e) => e.fmt(f),
            Self::RelayPool(e) => e.fmt(f),
            Self::Database(e) => e.fmt(f),
            Self::Gossip(e) => e.fmt(f),
            Self::Json(e) => e.fmt(f),
            Self::SignerNotConfigured => f.write_str("signer not configured"),
            Self::GossipNotConfigured => f.write_str("gossip not configured"),
            Self::GossipFiltersEmpty => {
                f.write_str("gossip broken down filters are empty")
            }
            Self::PrivateMsgRelaysNotFound => f.write_str("Private message relays not found. The user is not ready to receive private messages."),
        }
    }
}

impl From<nostr::error::Error> for Error {
    fn from(e: nostr::error::Error) -> Self {
        Self::Protocol(e)
    }
}

impl From<relay::Error> for Error {
    fn from(e: relay::Error) -> Self {
        Self::Relay(e)
    }
}

impl From<pool::Error> for Error {
    fn from(e: pool::Error) -> Self {
        Self::RelayPool(e)
    }
}

impl From<nostr_database::error::Error> for Error {
    fn from(e: nostr_database::error::Error) -> Self {
        Self::Database(e)
    }
}

impl From<GossipError> for Error {
    fn from(e: GossipError) -> Self {
        Self::Gossip(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
