// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;
use std::sync::Arc;

use nostr::{JsonUtil, SubscriptionId};
use uniffi::Enum;

use crate::error::Result;
use crate::protocol::event::{Event, EventId};

#[derive(Enum)]
pub enum RelayMessage {
    Evnt {
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
        message: String,
    },
}

impl From<nostr::RelayMessage> for RelayMessage {
    fn from(value: nostr::RelayMessage) -> Self {
        match value {
            nostr::RelayMessage::Event {
                subscription_id,
                event,
            } => Self::Evnt {
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
            nostr::RelayMessage::Notice(message) => Self::Notice { message },
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
                message,
            } => Self::NegErr {
                subscription_id: subscription_id.to_string(),
                message,
            },
        }
    }
}

impl From<RelayMessage> for nostr::RelayMessage {
    fn from(value: RelayMessage) -> Self {
        match value {
            RelayMessage::Evnt {
                subscription_id,
                event,
            } => Self::Event {
                subscription_id: SubscriptionId::new(subscription_id),
                event: Box::new(event.as_ref().deref().clone()),
            },
            RelayMessage::Closed {
                subscription_id,
                message,
            } => Self::Closed {
                subscription_id: SubscriptionId::new(subscription_id),
                message,
            },
            RelayMessage::Notice { message } => Self::Notice(message),
            RelayMessage::EndOfStoredEvents { subscription_id } => {
                Self::eose(SubscriptionId::new(subscription_id))
            }
            RelayMessage::Ok {
                event_id,
                status,
                message,
            } => Self::Ok {
                event_id: **event_id,
                status,
                message,
            },
            RelayMessage::Auth { challenge } => Self::Auth { challenge },
            RelayMessage::Count {
                subscription_id,
                count,
            } => Self::Count {
                subscription_id: SubscriptionId::new(subscription_id),
                count: count as usize,
            },
            RelayMessage::NegMsg {
                subscription_id,
                message,
            } => Self::NegMsg {
                subscription_id: SubscriptionId::new(subscription_id),
                message,
            },
            RelayMessage::NegErr {
                subscription_id,
                message,
            } => Self::NegErr {
                subscription_id: SubscriptionId::new(subscription_id),
                message,
            },
        }
    }
}

/// Deserialize `RelayMessage` from JSON string
///
/// **This method doesn't verify the event signature!**
#[uniffi::export]
pub fn relay_message_from_json(json: &str) -> Result<RelayMessage> {
    let msg: nostr::RelayMessage = nostr::RelayMessage::from_json(json)?;
    Ok(msg.into())
}

#[uniffi::export]
pub fn relay_message_as_json(msg: RelayMessage) -> Result<String> {
    let msg: nostr::RelayMessage = msg.into();
    Ok(msg.try_as_json()?)
}
