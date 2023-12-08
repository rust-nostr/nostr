// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::SubscriptionId;
use uniffi::Enum;

use crate::{Event, Filter};

#[derive(Enum)]
pub enum ClientMessage {
    Event {
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
}

impl From<ClientMessage> for nostr::ClientMessage {
    fn from(value: ClientMessage) -> Self {
        match value {
            ClientMessage::Event { event } => Self::Event(Box::new(event.as_ref().deref().clone())),
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
        }
    }
}
