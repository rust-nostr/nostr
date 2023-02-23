// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Messages

pub mod client;
pub mod relay;
pub mod subscription;

pub use self::client::ClientMessage;
pub use self::relay::RelayMessage;
pub use self::subscription::{Filter, SubscriptionId};

#[allow(deprecated)]
pub use self::subscription::SubscriptionFilter;

/// Messages error
#[derive(Debug, thiserror::Error)]
pub enum MessageHandleError {
    /// Invalid message format
    #[error("Message has an invalid format")]
    InvalidMessageFormat,
    /// Impossible to deserialize message
    #[error("Json deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    /// Event error
    #[error(transparent)]
    Event(#[from] crate::event::Error),
}
