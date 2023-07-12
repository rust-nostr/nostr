// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::{Event, Filter, SubscriptionId};

use crate::NostrError;

pub enum ClientMessage {
    Ev {
        event: String,
    },
    Req {
        subscription_id: String,
        filters: Vec<String>,
    },
    Count {
        subscription_id: String,
        filters: Vec<String>,
    },
    Close {
        subscription_id: String,
    },
    Auth {
        event: String,
    },
}

impl TryFrom<ClientMessage> for nostr::ClientMessage {
    type Error = NostrError;
    fn try_from(value: ClientMessage) -> Result<Self, Self::Error> {
        match value {
            ClientMessage::Ev { event } => Ok(Self::Event(Box::new(Event::from_json(event)?))),
            ClientMessage::Req {
                subscription_id,
                filters,
            } => {
                let mut f = Vec::new();
                for filter in filters.into_iter() {
                    f.push(Filter::from_json(filter)?);
                }
                Ok(Self::Req {
                    subscription_id: SubscriptionId::new(subscription_id),
                    filters: f,
                })
            }
            ClientMessage::Count {
                subscription_id,
                filters,
            } => {
                let mut f = Vec::new();
                for filter in filters.into_iter() {
                    f.push(Filter::from_json(filter)?);
                }
                Ok(Self::Count {
                    subscription_id: SubscriptionId::new(subscription_id),
                    filters: f,
                })
            }
            ClientMessage::Close { subscription_id } => {
                Ok(Self::Close(SubscriptionId::new(subscription_id)))
            }
            ClientMessage::Auth { event } => Ok(Self::Auth(Box::new(Event::from_json(event)?))),
        }
    }
}
