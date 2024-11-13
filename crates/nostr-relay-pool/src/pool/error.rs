// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::convert::Infallible;

use async_utility::thread;
use nostr::types::url;
use nostr_database::DatabaseError;
use thiserror::Error;

use crate::relay;

/// Relay Pool error
#[derive(Debug, Error)]
pub enum Error {
    /// Url parse error
    #[error("impossible to parse URL: {0}")]
    Url(#[from] url::ParseError),
    /// Relay error
    #[error(transparent)]
    Relay(#[from] relay::Error),
    /// Database error
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// Thread error
    #[error(transparent)]
    Thread(#[from] thread::Error),
    /// No relays
    #[error("too many relays (limit: {limit})")]
    TooManyRelays {
        /// Max numer allowed
        limit: usize,
    },
    /// No relays
    #[error("no relays")]
    NoRelays,
    /// No relays specified
    #[error("no relays specified")]
    NoRelaysSpecified,
    /// Msg not sent
    #[error("message not sent")]
    MsgNotSent,
    /// Msgs not sent
    #[error("messages not sent")]
    MsgsNotSent,
    /// Event/s not published
    #[error("event/s not published")]
    EventNotPublished,
    /// Not subscribed
    #[error("not subscribed")]
    NotSubscribed,
    /// Negentropy reconciliation failed
    #[error("negentropy reconciliation failed")]
    NegentropyReconciliationFailed,
    /// Relay not found
    #[error("relay not found")]
    RelayNotFound,
    /// Relay Pool is shutdown
    #[error("Relay Pool is shutdown")]
    Shutdown,
    /// Notification Handler error
    #[error("notification handler error: {0}")]
    Handler(String),
    /// Infallible
    #[error(transparent)]
    Infallible(#[from] Infallible),
}
