// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr::{ClientMessage, JsonUtil, SubscriptionId};
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::JsEvent;
use crate::types::JsFilter;

#[wasm_bindgen(js_name = ClientMessage)]
pub struct JsClientMessage {
    inner: ClientMessage,
}

impl Deref for JsClientMessage {
    type Target = ClientMessage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<ClientMessage> for JsClientMessage {
    fn from(inner: ClientMessage) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = ClientMessage)]
impl JsClientMessage {
    /// Create new `EVENT` message
    pub fn event(event: &JsEvent) -> Self {
        Self {
            inner: ClientMessage::event(event.deref().clone()),
        }
    }

    /// Create new `REQ` message
    pub fn req(subscription_id: &str, filters: Vec<JsFilter>) -> Self {
        Self {
            inner: ClientMessage::req(
                SubscriptionId::new(subscription_id),
                filters.into_iter().map(|f| f.into()).collect(),
            ),
        }
    }

    /// Create new `COUNT` message
    pub fn count(subscription_id: &str, filters: Vec<JsFilter>) -> Self {
        Self {
            inner: ClientMessage::count(
                SubscriptionId::new(subscription_id),
                filters.into_iter().map(|f| f.into()).collect(),
            ),
        }
    }

    /// Create new `CLOSE` message
    pub fn close(subscription_id: &str) -> Self {
        Self {
            inner: ClientMessage::close(SubscriptionId::new(subscription_id)),
        }
    }

    /// Create new `AUTH` message
    pub fn auth(event: &JsEvent) -> Self {
        Self {
            inner: ClientMessage::auth(event.deref().clone()),
        }
    }

    /// Deserialize `ClientMessage` from JSON string
    ///
    /// **This method NOT verify the event signature!**
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<JsClientMessage> {
        Ok(Self {
            inner: ClientMessage::from_json(json).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = asJson)]
    pub fn as_json(&self) -> Result<String> {
        self.inner.try_as_json().map_err(into_err)
    }
}
