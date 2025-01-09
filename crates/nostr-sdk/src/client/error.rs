// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use nostr::prelude::*;
use nostr::serde_json;
use nostr_database::prelude::*;
use nostr_relay_pool::__private::SharedStateError;
use nostr_relay_pool::prelude::*;
#[cfg(feature = "nip57")]
use nostr_zapper::ZapperError;

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
    /// Zapper error
    #[cfg(feature = "nip57")]
    Zapper(ZapperError),
    /// [`EventBuilder`] error
    EventBuilder(builder::Error),
    /// Json error
    Json(serde_json::Error),
    /// Shared state error
    SharedState(SharedStateError),
    /// NIP57 error
    #[cfg(feature = "nip57")]
    NIP57(nip57::Error),
    /// LNURL Pay
    #[cfg(feature = "nip57")]
    LnUrlPay(lnurl_pay::Error),
    /// NIP59
    #[cfg(feature = "nip59")]
    NIP59(nip59::Error),
    /// Zapper not configured
    #[cfg(feature = "nip57")]
    ZapperNotConfigured,
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
            #[cfg(feature = "nip57")]
            Self::Zapper(e) => write!(f, "{e}"),
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::SharedState(e) => write!(f, "{e}"),
            #[cfg(feature = "nip57")]
            Self::NIP57(e) => write!(f, "{e}"),
            #[cfg(feature = "nip57")]
            Self::LnUrlPay(e) => write!(f, "{e}"),
            #[cfg(feature = "nip59")]
            Self::NIP59(e) => write!(f, "{e}"),
            #[cfg(feature = "nip57")]
            Self::ZapperNotConfigured => {
                write!(f, "zapper not configured")
            }
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

#[cfg(feature = "nip57")]
impl From<ZapperError> for Error {
    fn from(e: ZapperError) -> Self {
        Self::Zapper(e)
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

#[cfg(feature = "nip57")]
impl From<nip57::Error> for Error {
    fn from(e: nip57::Error) -> Self {
        Self::NIP57(e)
    }
}

#[cfg(feature = "nip57")]
impl From<lnurl_pay::Error> for Error {
    fn from(e: lnurl_pay::Error) -> Self {
        Self::LnUrlPay(e)
    }
}

#[cfg(feature = "nip59")]
impl From<nip59::Error> for Error {
    fn from(e: nip59::Error) -> Self {
        Self::NIP59(e)
    }
}
