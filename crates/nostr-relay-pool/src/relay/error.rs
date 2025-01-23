// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::time::Duration;

use nostr::event;
use nostr::event::builder;
use nostr::message::MessageHandleError;
use nostr_database::DatabaseError;
use tokio::sync::{broadcast, SetError};

use crate::shared::SharedStateError;
use crate::transport::error::TransportError;
use crate::RelayPoolNotification;

/// Relay error
#[derive(Debug)]
pub enum Error {
    /// Transport error
    Transport(TransportError),
    /// Shared state error
    SharedState(SharedStateError),
    /// MessageHandle error
    MessageHandle(MessageHandleError),
    /// Event error
    Event(event::Error),
    /// Event Builder error
    EventBuilder(builder::Error),
    /// Partial Event error
    PartialEvent(event::partial::Error),
    /// Negentropy error
    Negentropy(negentropy::Error),
    /// Negentropy error
    NegentropyDeprecated(negentropy_deprecated::Error),
    /// Database error
    Database(DatabaseError),
    /// OnceCell error
    SetPoolNotificationSender(SetError<broadcast::Sender<RelayPoolNotification>>),
    /// Generic timeout
    Timeout,
    /// Not replied to ping
    NotRepliedToPing,
    /// Can't parse pong
    CantParsePong,
    /// Pong not match
    PongNotMatch {
        /// Expected nonce
        expected: u64,
        /// Received nonce
        received: u64,
    },
    /// Message response timeout
    CantSendChannelMessage {
        /// Name of channel
        channel: String,
    },
    /// Relay not ready
    NotReady,
    /// Relay not connected
    NotConnected,
    /// Received shutdown
    ReceivedShutdown,
    /// Relay message
    RelayMessage(String),
    /// Batch messages empty
    BatchMessagesEmpty,
    /// Read actions disabled
    ReadDisabled,
    /// Write actions disabled
    WriteDisabled,
    /// Filters empty
    FiltersEmpty,
    /// Negentropy not supported
    NegentropyNotSupported,
    /// Unknown negentropy error
    UnknownNegentropyError,
    /// Relay message too large
    RelayMessageTooLarge {
        /// Message size
        size: usize,
        /// Max message size
        max_size: usize,
    },
    /// Event too large
    EventTooLarge {
        /// Event size
        size: usize,
        /// Max event size
        max_size: usize,
    },
    /// Too many tags
    TooManyTags {
        /// Tags num
        size: usize,
        /// Max tags num
        max_size: usize,
    },
    /// Event expired
    EventExpired,
    /// POW difficulty too low
    PowDifficultyTooLow {
        /// Min. difficulty
        min: u8,
    },
    /// Notification Handler error
    Handler(String),
    /// Max latency exceeded
    MaximumLatencyExceeded {
        /// Max
        max: Duration,
        /// Current
        current: Duration,
    },
    /// Auth failed
    AuthenticationFailed,
    /// Premature exit
    PrematureExit,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "{e}"),
            Self::SharedState(e) => write!(f, "{e}"),
            Self::MessageHandle(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::PartialEvent(e) => write!(f, "{e}"),
            Self::Negentropy(e) => write!(f, "{e}"),
            Self::NegentropyDeprecated(e) => write!(f, "{e}"),
            Self::Database(e) => write!(f, "{e}"),
            Self::SetPoolNotificationSender(e) => write!(f, "{e}"),
            Self::Timeout => write!(f, "timeout"),
            Self::NotRepliedToPing => write!(f, "not replied to ping"),
            Self::CantParsePong => write!(f, "can't parse pong"),
            Self::PongNotMatch { expected, received } => write!(
                f,
                "pong not match: expected={expected}, received={received}"
            ),
            Self::CantSendChannelMessage { channel } => {
                write!(f, "can't send message to the '{channel}' channel")
            }
            Self::NotReady => write!(f, "relay is initialized but not ready"),
            Self::NotConnected => write!(f, "relay not connected"),
            Self::ReceivedShutdown => write!(f, "received shutdown"),
            Self::RelayMessage(message) => write!(f, "{message}"),
            Self::BatchMessagesEmpty => write!(f, "can't batch empty list of messages"),
            Self::ReadDisabled => write!(f, "read actions are disabled"),
            Self::WriteDisabled => write!(f, "write actions are disabled"),
            Self::FiltersEmpty => write!(f, "filters empty"),
            Self::NegentropyNotSupported => write!(f, "negentropy not supported"),
            Self::UnknownNegentropyError => write!(f, "unknown negentropy error"),
            Self::RelayMessageTooLarge { size, max_size } => write!(
                f,
                "Received message too large: size={size}, max_size={max_size}"
            ),
            Self::EventTooLarge { size, max_size } => write!(
                f,
                "Received event too large: size={size}, max_size={max_size}"
            ),
            Self::TooManyTags { size, max_size } => write!(
                f,
                "Received event with too many tags: tags={size}, max_tags={max_size}"
            ),
            Self::EventExpired => write!(f, "event expired"),
            Self::PowDifficultyTooLow { min } => write!(f, "POW difficulty too low (min. {min})"),
            Self::Handler(e) => write!(f, "{e}"),
            Self::MaximumLatencyExceeded { max, current } => write!(
                f,
                "Maximum latency exceeded: max={}ms, current={}ms",
                max.as_millis(),
                current.as_millis()
            ),
            Self::AuthenticationFailed => write!(f, "authentication failed"),
            Self::PrematureExit => write!(f, "premature exit"),
        }
    }
}

impl From<TransportError> for Error {
    fn from(e: TransportError) -> Self {
        Self::Transport(e)
    }
}

impl From<SharedStateError> for Error {
    fn from(e: SharedStateError) -> Self {
        Self::SharedState(e)
    }
}

impl From<MessageHandleError> for Error {
    fn from(e: MessageHandleError) -> Self {
        Self::MessageHandle(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<builder::Error> for Error {
    fn from(e: builder::Error) -> Self {
        Self::EventBuilder(e)
    }
}

impl From<event::partial::Error> for Error {
    fn from(e: event::partial::Error) -> Self {
        Self::PartialEvent(e)
    }
}

impl From<negentropy::Error> for Error {
    fn from(e: negentropy::Error) -> Self {
        Self::Negentropy(e)
    }
}

impl From<negentropy_deprecated::Error> for Error {
    fn from(e: negentropy_deprecated::Error) -> Self {
        Self::NegentropyDeprecated(e)
    }
}

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Self::Database(e)
    }
}

impl From<SetError<broadcast::Sender<RelayPoolNotification>>> for Error {
    fn from(e: SetError<broadcast::Sender<RelayPoolNotification>>) -> Self {
        Self::SetPoolNotificationSender(e)
    }
}
