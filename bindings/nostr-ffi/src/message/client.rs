// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::{JsonUtil, SubscriptionId};
use uniffi::{Enum, Object};

use crate::error::Result;
use crate::types::filter::Filter;
use crate::Event;

/// Messages sent by clients, received by relays
#[derive(Enum)]
pub enum ClientMessageEnum {
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

impl From<ClientMessageEnum> for nostr::ClientMessage {
    fn from(value: ClientMessageEnum) -> Self {
        match value {
            ClientMessageEnum::EventMsg { event } => {
                Self::Event(Box::new(event.as_ref().deref().clone()))
            }
            ClientMessageEnum::Req {
                subscription_id,
                filters,
            } => Self::Req {
                subscription_id: SubscriptionId::new(subscription_id),
                filters: filters
                    .into_iter()
                    .map(|f| f.as_ref().deref().clone())
                    .collect(),
            },
            ClientMessageEnum::Count {
                subscription_id,
                filters,
            } => Self::Count {
                subscription_id: SubscriptionId::new(subscription_id),
                filters: filters
                    .into_iter()
                    .map(|f| f.as_ref().deref().clone())
                    .collect(),
            },
            ClientMessageEnum::Close { subscription_id } => {
                Self::Close(SubscriptionId::new(subscription_id))
            }
            ClientMessageEnum::Auth { event } => {
                Self::Auth(Box::new(event.as_ref().deref().clone()))
            }
            ClientMessageEnum::NegOpen {
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
            ClientMessageEnum::NegMsg {
                subscription_id,
                message,
            } => Self::NegMsg {
                subscription_id: SubscriptionId::new(subscription_id),
                message,
            },
            ClientMessageEnum::NegClose { subscription_id } => Self::NegClose {
                subscription_id: SubscriptionId::new(subscription_id),
            },
        }
    }
}

impl From<nostr::ClientMessage> for ClientMessageEnum {
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

#[derive(Debug, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
pub struct ClientMessage {
    inner: nostr::ClientMessage,
}

impl Deref for ClientMessage {
    type Target = nostr::ClientMessage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr::ClientMessage> for ClientMessage {
    fn from(inner: nostr::ClientMessage) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl ClientMessage {
    /// Create new `EVENT` message
    #[uniffi::constructor]
    pub fn event(event: &Event) -> Self {
        Self {
            inner: nostr::ClientMessage::event(event.deref().clone()),
        }
    }

    /// Create new `REQ` message
    #[uniffi::constructor]
    pub fn req(subscription_id: &str, filters: Vec<Arc<Filter>>) -> Self {
        Self {
            inner: nostr::ClientMessage::req(
                SubscriptionId::new(subscription_id),
                filters
                    .into_iter()
                    .map(|f| f.as_ref().deref().clone())
                    .collect(),
            ),
        }
    }

    /// Create new `COUNT` message
    #[uniffi::constructor]
    pub fn count(subscription_id: &str, filters: Vec<Arc<Filter>>) -> Self {
        Self {
            inner: nostr::ClientMessage::count(
                SubscriptionId::new(subscription_id),
                filters
                    .into_iter()
                    .map(|f| f.as_ref().deref().clone())
                    .collect(),
            ),
        }
    }

    /// Create new `CLOSE` message
    #[uniffi::constructor]
    pub fn close(subscription_id: &str) -> Self {
        Self {
            inner: nostr::ClientMessage::close(SubscriptionId::new(subscription_id)),
        }
    }

    /// Create new `AUTH` message
    #[uniffi::constructor]
    pub fn auth(event: &Event) -> Self {
        Self {
            inner: nostr::ClientMessage::auth(event.deref().clone()),
        }
    }

    /// Deserialize `ClientMessage` from JSON string
    ///
    /// **This method NOT verify the event signature!**
    #[uniffi::constructor]
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(Self {
            inner: nostr::ClientMessage::from_json(json)?,
        })
    }

    /// Convert `ClientMessageEnum` to `ClientMessage`
    #[uniffi::constructor]
    pub fn from_enum(e: ClientMessageEnum) -> Self {
        Self { inner: e.into() }
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }

    /// Clone `ClientMessage` and convert it to `ClientMessageEnum`
    pub fn as_enum(&self) -> ClientMessageEnum {
        self.inner.clone().into()
    }
}
