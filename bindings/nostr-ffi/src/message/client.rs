// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::SubscriptionId;
use uniffi::Enum;

use crate::{Event, Filter};

/// Messages sent by clients, received by relays
#[derive(Enum)]
pub enum ClientMessage {
    EventMsg {
        event: Arc<Event>,
    },
    Req {
        subscription_id: String,
        filters: Vec<Arc<Filter>>,
    },
    Count {
        subscription_id: String,
        filters: Vec<Arc<Filter>>,
    },
    Close {
        subscription_id: String,
    },
    Auth {
        event: Arc<Event>,
    },
    /// Negentropy Open
    NegOpen {
        subscription_id: String,
        filter: Arc<Filter>,
        /// ID size (MUST be between 8 and 32, inclusive)
        id_size: u8,
        initial_message: String,
    },
    /// Negentropy Message
    NegMsg {
        subscription_id: String,
        message: String,
    },
    /// Negentropy Close
    NegClose {
        subscription_id: String,
    },
}

impl From<ClientMessage> for nostr::ClientMessage {
    fn from(value: ClientMessage) -> Self {
        match value {
            ClientMessage::EventMsg { event } => {
                Self::Event(Box::new(event.as_ref().deref().clone()))
            }
            ClientMessage::Req {
                subscription_id,
                filters,
            } => Self::Req {
                subscription_id: SubscriptionId::new(subscription_id),
                filters: filters
                    .into_iter()
                    .map(|f| f.as_ref().deref().clone())
                    .collect(),
            },
            ClientMessage::Count {
                subscription_id,
                filters,
            } => Self::Count {
                subscription_id: SubscriptionId::new(subscription_id),
                filters: filters
                    .into_iter()
                    .map(|f| f.as_ref().deref().clone())
                    .collect(),
            },
            ClientMessage::Close { subscription_id } => {
                Self::Close(SubscriptionId::new(subscription_id))
            }
            ClientMessage::Auth { event } => Self::Auth(Box::new(event.as_ref().deref().clone())),
            ClientMessage::NegOpen {
                subscription_id,
                filter,
                id_size,
                initial_message,
            } => Self::NegOpen {
                subscription_id: SubscriptionId::new(subscription_id),
                filter: Box::new(filter.as_ref().deref().clone()),
                id_size,
                initial_message,
            },
            ClientMessage::NegMsg {
                subscription_id,
                message,
            } => Self::NegMsg {
                subscription_id: SubscriptionId::new(subscription_id),
                message,
            },
            ClientMessage::NegClose { subscription_id } => Self::NegClose {
                subscription_id: SubscriptionId::new(subscription_id),
            },
        }
    }
}

impl From<nostr::ClientMessage> for ClientMessage {
    fn from(value: nostr::ClientMessage) -> Self {
        match value {
            nostr::ClientMessage::Event(event) => Self::EventMsg {
                event: Arc::new(event.as_ref().to_owned().into()),
            },
            nostr::ClientMessage::Req {
                subscription_id,
                filters,
            } => Self::Req {
                subscription_id: subscription_id.to_string(),
                filters: filters.into_iter().map(|f| Arc::new(f.into())).collect(),
            },
            nostr::ClientMessage::Count {
                subscription_id,
                filters,
            } => Self::Count {
                subscription_id: subscription_id.to_string(),
                filters: filters.into_iter().map(|f| Arc::new(f.into())).collect(),
            },
            nostr::ClientMessage::Close(subscription_id) => Self::Close {
                subscription_id: subscription_id.to_string(),
            },
            nostr::ClientMessage::Auth(event) => Self::Auth {
                event: Arc::new(event.as_ref().to_owned().into()),
            },
            nostr::ClientMessage::NegOpen {
                subscription_id,
                filter,
                id_size,
                initial_message,
            } => Self::NegOpen {
                subscription_id: subscription_id.to_string(),
                filter: Arc::new(filter.as_ref().to_owned().into()),
                id_size,
                initial_message,
            },
            nostr::ClientMessage::NegMsg {
                subscription_id,
                message,
            } => Self::NegMsg {
                subscription_id: subscription_id.to_string(),
                message,
            },
            nostr::ClientMessage::NegClose { subscription_id } => Self::NegClose {
                subscription_id: subscription_id.to_string(),
            },
        }
    }
}
