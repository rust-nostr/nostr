// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Messages

use core::fmt;

pub mod client;
pub mod relay;
pub mod subscription;

pub use self::client::ClientMessage;
pub use self::relay::{RawRelayMessage, RelayMessage};
pub use self::subscription::{
    Alphabet, Filter, FiltersMatchEvent, GenericTagValue, SubscriptionId,
};
use crate::event;

/// Messages error
#[derive(Debug)]
pub enum MessageHandleError {
    /// Invalid message format
    InvalidMessageFormat,
    /// Impossible to deserialize message
    Json(serde_json::Error),
    /// Event ID error
    EventId(event::id::Error),
    /// Event error
    Event(event::Error),
    /// Empty message
    EmptyMsg,
}

#[cfg(feature = "std")]
impl std::error::Error for MessageHandleError {}

impl fmt::Display for MessageHandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMessageFormat => write!(f, "Message has an invalid format"),
            Self::Json(e) => write!(f, "Json deserialization failed: {e}"),
            Self::EventId(e) => write!(f, "EventId: {e}"),
            Self::Event(e) => write!(f, "Event: {e}"),
            Self::EmptyMsg => write!(f, "Received empty message"),
        }
    }
}

impl From<serde_json::Error> for MessageHandleError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<event::id::Error> for MessageHandleError {
    fn from(e: event::id::Error) -> Self {
        Self::EventId(e)
    }
}

impl From<event::Error> for MessageHandleError {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}
