// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Messages

use core::fmt;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use core::error::Error as StdError;

#[cfg(feature = "std")]
use std::error::Error as StdError;

pub mod client;
pub mod relay;
pub mod subscription;

pub use self::client::ClientMessage;
pub use self::relay::RelayMessage;
pub use self::subscription::{Filter, SubscriptionId};

/// Messages error
#[derive(Debug)]
pub enum MessageHandleError {
    /// Invalid message format
    InvalidMessageFormat,
    /// Impossible to deserialize message
    Json(serde_json::Error),
    /// Event error
    Event(crate::event::Error),
}

impl StdError for MessageHandleError {}

impl fmt::Display for MessageHandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMessageFormat => write!(f, "Message has an invalid format"),
            Self::Json(e) => write!(f, "Json deserialization failed: {e}"),
            Self::Event(e) => write!(f, "{e}"),
        }
    }
}

impl From<serde_json::Error> for MessageHandleError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<crate::event::Error> for MessageHandleError {
    fn from(e: crate::event::Error) -> Self {
        Self::Event(e)
    }
}
