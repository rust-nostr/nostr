// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::{JsonUtil, RelayMessage, SubscriptionId};
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::{JsEvent, JsEventId};

#[wasm_bindgen(js_name = RelayMessage)]
pub struct JsRelayMessage {
    inner: RelayMessage,
}

impl From<RelayMessage> for JsRelayMessage {
    fn from(inner: RelayMessage) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = RelayMessage)]
impl JsRelayMessage {
    /// Create new `EVENT` message
    pub fn event(subscription_id: &str, event: &JsEvent) -> Self {
        Self {
            inner: RelayMessage::event(SubscriptionId::new(subscription_id), event.deref().clone()),
        }
    }

    /// Create new `NOTICE` message
    pub fn notice(message: &str) -> Self {
        Self {
            inner: RelayMessage::notice(message),
        }
    }

    /// Create new `CLOSED` message
    pub fn closed(subscription_id: &str, message: &str) -> Self {
        Self {
            inner: RelayMessage::closed(SubscriptionId::new(subscription_id), message),
        }
    }

    /// Create new `EOSE` message
    pub fn eose(subscription_id: &str) -> Self {
        Self {
            inner: RelayMessage::eose(SubscriptionId::new(subscription_id)),
        }
    }

    /// Create new `OK` message
    pub fn ok(event_id: &JsEventId, status: bool, message: &str) -> Self {
        Self {
            inner: RelayMessage::ok(**event_id, status, message),
        }
    }

    /// Create new `AUTH` message
    pub fn auth(challenge: &str) -> Self {
        Self {
            inner: RelayMessage::auth(challenge),
        }
    }

    /// Create new `EVENT` message
    pub fn count(subscription_id: &str, count: f64) -> Self {
        Self {
            inner: RelayMessage::count(SubscriptionId::new(subscription_id), count as usize),
        }
    }

    /// Deserialize `RelayMessage` from JSON string
    ///
    /// **This method NOT verify the event signature!**
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<JsRelayMessage> {
        Ok(Self {
            inner: RelayMessage::from_json(json).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> Result<String> {
        self.inner.try_as_json().map_err(into_err)
    }
}
