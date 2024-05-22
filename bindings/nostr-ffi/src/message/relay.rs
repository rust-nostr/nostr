// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use std::sync::Arc;

use nostr::message::relay::NegentropyErrorCode;
use nostr::{JsonUtil, SubscriptionId};
use uniffi::{Enum, Object};

use crate::error::Result;
use crate::{Event, EventId};

#[derive(Enum)]
pub enum RelayMessageEnum {
    EventMsg {
        subscription_id: String,
        event: Arc<Event>,
    },
    Ok {
        event_id: Arc<EventId>,
        status: bool,
        message: String,
    },
    EndOfStoredEvents {
        subscription_id: String,
    },
    Notice {
        message: String,
    },
    Closed {
        subscription_id: String,
        message: String,
    },
    Auth {
        challenge: String,
    },
    Count {
        subscription_id: String,
        count: u64,
    },
    NegMsg {
        subscription_id: String,
        message: String,
    },
    NegErr {
        subscription_id: String,
        code: String,
    },
}

impl From<nostr::RelayMessage> for RelayMessageEnum {
    fn from(value: nostr::RelayMessage) -> Self {
        match value {
            nostr::RelayMessage::Event {
                subscription_id,
                event,
            } => Self::EventMsg {
                subscription_id: subscription_id.to_string(),
                event: Arc::new(event.as_ref().clone().into()),
            },
            nostr::RelayMessage::Closed {
                subscription_id,
                message,
            } => Self::Closed {
                subscription_id: subscription_id.to_string(),
                message,
            },
            nostr::RelayMessage::Notice { message } => Self::Notice { message },
            nostr::RelayMessage::EndOfStoredEvents(sub_id) => Self::EndOfStoredEvents {
                subscription_id: sub_id.to_string(),
            },
            nostr::RelayMessage::Ok {
                event_id,
                status,
                message,
            } => Self::Ok {
                event_id: Arc::new(event_id.into()),
                status,
                message,
            },
            nostr::RelayMessage::Auth { challenge } => Self::Auth { challenge },
            nostr::RelayMessage::Count {
                subscription_id,
                count,
            } => Self::Count {
                subscription_id: subscription_id.to_string(),
                count: count as u64,
            },
            nostr::RelayMessage::NegMsg {
                subscription_id,
                message,
            } => Self::NegMsg {
                subscription_id: subscription_id.to_string(),
                message,
            },
            nostr::RelayMessage::NegErr {
                subscription_id,
                code,
            } => Self::NegErr {
                subscription_id: subscription_id.to_string(),
                code: code.to_string(),
            },
        }
    }
}

impl From<RelayMessageEnum> for nostr::RelayMessage {
    fn from(value: RelayMessageEnum) -> Self {
        match value {
            RelayMessageEnum::EventMsg {
                subscription_id,
                event,
            } => Self::Event {
                subscription_id: SubscriptionId::new(subscription_id),
                event: Box::new(event.as_ref().deref().clone()),
            },
            RelayMessageEnum::Closed {
                subscription_id,
                message,
            } => Self::Closed {
                subscription_id: SubscriptionId::new(subscription_id),
                message,
            },
            RelayMessageEnum::Notice { message } => Self::Notice { message },
            RelayMessageEnum::EndOfStoredEvents { subscription_id } => {
                Self::eose(SubscriptionId::new(subscription_id))
            }
            RelayMessageEnum::Ok {
                event_id,
                status,
                message,
            } => Self::Ok {
                event_id: **event_id,
                status,
                message,
            },
            RelayMessageEnum::Auth { challenge } => Self::Auth { challenge },
            RelayMessageEnum::Count {
                subscription_id,
                count,
            } => Self::Count {
                subscription_id: SubscriptionId::new(subscription_id),
                count: count as usize,
            },
            RelayMessageEnum::NegMsg {
                subscription_id,
                message,
            } => Self::NegMsg {
                subscription_id: SubscriptionId::new(subscription_id),
                message,
            },
            RelayMessageEnum::NegErr {
                subscription_id,
                code,
            } => Self::NegErr {
                subscription_id: SubscriptionId::new(subscription_id),
                code: NegentropyErrorCode::from(code),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct RelayMessage {
    inner: nostr::RelayMessage,
}

impl From<nostr::RelayMessage> for RelayMessage {
    fn from(inner: nostr::RelayMessage) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl RelayMessage {
    /// Create new `EVENT` message
    #[uniffi::constructor]
    pub fn event(subscription_id: &str, event: &Event) -> Self {
        Self {
            inner: nostr::RelayMessage::event(
                SubscriptionId::new(subscription_id),
                event.deref().clone(),
            ),
        }
    }

    /// Create new `NOTICE` message
    #[uniffi::constructor]
    pub fn notice(message: &str) -> Self {
        Self {
            inner: nostr::RelayMessage::notice(message),
        }
    }

    /// Create new `CLOSED` message
    #[uniffi::constructor]
    pub fn closed(subscription_id: &str, message: &str) -> Self {
        Self {
            inner: nostr::RelayMessage::closed(SubscriptionId::new(subscription_id), message),
        }
    }

    /// Create new `EOSE` message
    #[uniffi::constructor]
    pub fn eose(subscription_id: &str) -> Self {
        Self {
            inner: nostr::RelayMessage::eose(SubscriptionId::new(subscription_id)),
        }
    }

    /// Create new `OK` message
    #[uniffi::constructor]
    pub fn ok(event_id: &EventId, status: bool, message: &str) -> Self {
        Self {
            inner: nostr::RelayMessage::ok(**event_id, status, message),
        }
    }

    /// Create new `AUTH` message
    #[uniffi::constructor]
    pub fn auth(challenge: &str) -> Self {
        Self {
            inner: nostr::RelayMessage::auth(challenge),
        }
    }

    /// Create new `EVENT` message
    #[uniffi::constructor]
    pub fn count(subscription_id: &str, count: f64) -> Self {
        Self {
            inner: nostr::RelayMessage::count(SubscriptionId::new(subscription_id), count as usize),
        }
    }

    /// Deserialize `RelayMessage` from JSON string
    ///
    /// **This method NOT verify the event signature!**
    #[uniffi::constructor]
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::RelayMessage::from_json(json)?,
        })
    }

    /// Convert `RelayMessageEnum` to `RelayMessage`
    #[uniffi::constructor]
    pub fn from_enum(e: RelayMessageEnum) -> Self {
        Self { inner: e.into() }
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }

    /// Clone `RelayMessage` and convert it to `RelayMessageEnum`
    pub fn as_enum(&self) -> RelayMessageEnum {
        self.inner.clone().into()
    }
}
