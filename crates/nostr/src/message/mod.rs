// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

pub mod client;
pub mod relay;
pub mod subscription;

pub use self::client::ClientMessage;
pub use self::relay::RelayMessage;
pub use self::subscription::SubscriptionFilter;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum MessageHandleError {
    #[error("Message has an invalid format")]
    InvalidMessageFormat,
    #[error("Json deserialization failed")]
    JsonDeserializationFailed,
}
