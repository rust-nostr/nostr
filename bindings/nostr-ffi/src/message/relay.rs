// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use uniffi::Enum;

use crate::{Event, EventId};

#[derive(Enum)]
pub enum RelayMessage {
    Event {
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

impl From<nostr::RelayMessage> for RelayMessage {
    fn from(value: nostr::RelayMessage) -> Self {
        match value {
            nostr::RelayMessage::Event {
                subscription_id,
                event,
            } => Self::Event {
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
