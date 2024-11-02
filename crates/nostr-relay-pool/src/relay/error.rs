// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::time::Duration;

use async_utility::thread;
use nostr::message::relay::NegentropyErrorCode;
use nostr::message::MessageHandleError;
use nostr::{event, EventId, Kind};
use nostr_database::DatabaseError;
use thiserror::Error;
use tokio::sync::{broadcast, SetError};

use crate::RelayPoolNotification;

/// [`Relay`](super::Relay) error
#[derive(Debug, Error)]
pub enum Error {
    /// MessageHandle error
    #[error(transparent)]
    MessageHandle(#[from] MessageHandleError),
    /// Event error
    #[error(transparent)]
    Event(#[from] event::Error),
    /// Partial Event error
    #[error(transparent)]
    PartialEvent(#[from] event::partial::Error),
    /// Negentropy error
    #[error(transparent)]
    Negentropy(#[from] negentropy::Error),
    /// Negentropy error
    #[error(transparent)]
    NegentropyDeprecated(#[from] negentropy_deprecated::Error),
    /// Database error
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// Thread error
    #[error(transparent)]
    Thread(#[from] thread::Error),
    /// OnceCell error
    #[error(transparent)]
    OnceCell(#[from] SetError<broadcast::Sender<RelayPoolNotification>>),
    /// WebSocket timeout
    #[error("WebSocket timeout")]
    WebSocketTimeout,
    /// Generic timeout
    #[error("timeout")]
    Timeout,
    /// Message response timeout
    #[error("Can't send message to the '{channel}' channel")]
    CantSendChannelMessage {
        /// Name of channel
        channel: String,
    },
    /// Relay not connected
    #[error("relay is initialized but not ready")]
    Initialized,
    /// Relay not connected
    #[error("relay not connected")]
    NotConnected,
    /// Relay not connected
    #[error("relay not connected (status changed)")]
    NotConnectedStatusChanged,
    /// Received shutdown
    #[error("received shutdown")]
    Shutdown,
    /// Event not published
    #[error("event not published: {0}")]
    EventNotPublished(String),
    /// No event is published
    #[error("events not published: {0:?}")]
    EventsNotPublished(HashMap<EventId, String>),
    /// Only some events
    #[error("partial publish: published={}, missing={}", published.len(), not_published.len())]
    PartialPublish {
        /// Published events
        published: Vec<EventId>,
        /// Not published events
        not_published: HashMap<EventId, String>,
    },
    /// Batch messages empty
    #[error("can't batch empty list of messages")]
    BatchMessagesEmpty,
    /// Read actions disabled
    #[error("read actions are disabled for this relay")]
    ReadDisabled,
    /// Write actions disabled
    #[error("write actions are disabled for this relay")]
    WriteDisabled,
    /// Filters empty
    #[error("filters empty")]
    FiltersEmpty,
    /// Reconciliation error
    #[error("negentropy reconciliation error: {0}")]
    NegentropyReconciliation(NegentropyErrorCode),
    /// Negentropy not supported
    #[error("negentropy (maybe) not supported")]
    NegentropyMaybeNotSupported,
    /// Unknown negentropy error
    #[error("unknown negentropy error")]
    UnknownNegentropyError,
    /// Relay message too large
    #[error("Received message too large: size={size}, max_size={max_size}")]
    RelayMessageTooLarge {
        /// Message size
        size: usize,
        /// Max message size
        max_size: usize,
    },
    /// Event too large
    #[error("Received event too large: size={size}, max_size={max_size}")]
    EventTooLarge {
        /// Event size
        size: usize,
        /// Max event size
        max_size: usize,
    },
    /// Too many tags
    #[error("Received event with too many tags: tags={size}, max_tags={max_size}")]
    TooManyTags {
        /// Tags num
        size: usize,
        /// Max tags num
        max_size: usize,
    },
    /// Event expired
    #[error("event expired")]
    EventExpired,
    /// POW difficulty too low
    #[error("POW difficulty too low (min. {min})")]
    PowDifficultyTooLow {
        /// Min. difficulty
        min: u8,
    },
    /// Unexpected kind
    #[error("Unexpected kind: expected={expected}, found={found}")]
    UnexpectedKind {
        /// Expected kind
        expected: Kind,
        /// Found kind
        found: Kind,
    },
    /// Notification Handler error
    #[error("notification handler error: {0}")]
    Handler(String),
    /// WebSocket error
    #[error("{0}")]
    WebSocket(Box<dyn std::error::Error + Send + Sync>),
    /// Max latency exceeded
    #[error("Maximum latency exceeded: max={}ms, current={}ms", max.as_millis(), current.as_millis())]
    MaximumLatencyExceeded {
        /// Max
        max: Duration,
        /// Current
        current: Duration,
    },
}

impl Error {
    #[inline]
    pub(super) fn websocket<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::WebSocket(Box::new(error))
    }
}
