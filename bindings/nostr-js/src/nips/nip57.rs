// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip57::{ZapRequestData, ZapType};
use nostr::UncheckedUrl;
use wasm_bindgen::prelude::*;

use super::nip01::JsCoordinate;
use crate::event::JsEventId;
use crate::key::JsPublicKey;

#[wasm_bindgen(js_name = ZapType)]
pub enum JsZapType {
    /// Public
    Public,
    /// Private
    Private,
    /// Anonymous
    Anonymous,
}

impl From<JsZapType> for ZapType {
    fn from(value: JsZapType) -> Self {
        match value {
            JsZapType::Public => Self::Public,
            JsZapType::Private => Self::Private,
            JsZapType::Anonymous => Self::Anonymous,
        }
    }
}

impl From<ZapType> for JsZapType {
    fn from(value: ZapType) -> Self {
        match value {
            ZapType::Public => Self::Public,
            ZapType::Private => Self::Private,
            ZapType::Anonymous => Self::Anonymous,
        }
    }
}

#[wasm_bindgen(js_name = ZapRequestData)]
pub struct JsZapRequestData {
    inner: ZapRequestData,
}

impl From<ZapRequestData> for JsZapRequestData {
    fn from(inner: ZapRequestData) -> Self {
        Self { inner }
    }
}

impl Deref for JsZapRequestData {
    type Target = ZapRequestData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = ZapRequestData)]
impl JsZapRequestData {
    pub fn new(
        public_key: &JsPublicKey,
        relays: Vec<String>,
        message: String,
        amount: Option<f64>,
        lnurl: Option<String>,
        event_id: Option<JsEventId>,
        event_coordinate: Option<JsCoordinate>,
    ) -> Self {
        Self {
            inner: ZapRequestData {
                public_key: **public_key,
                relays: relays.into_iter().map(|r| UncheckedUrl::from(&r)).collect(),
                message,
                amount: amount.map(|n| n as u64),
                lnurl,
                event_id: event_id.map(|e| e.deref().clone()),
                event_coordinate: event_coordinate.map(|e| e.deref().clone()),
            },
        }
    }

    #[wasm_bindgen(getter, js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    #[wasm_bindgen(getter)]
    pub fn relays(&self) -> Vec<String> {
        self.inner
            .relays
            .clone()
            .into_iter()
            .map(|url| url.to_string())
            .collect()
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.inner.message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> Option<f64> {
        self.inner.amount.map(|n| n as f64)
    }

    #[wasm_bindgen(getter)]
    pub fn lnurl(&self) -> Option<String> {
        self.inner.lnurl.clone()
    }

    #[wasm_bindgen(getter, js_name = eventID)]
    pub fn event_id(&self) -> Option<JsEventId> {
        self.inner.event_id.map(|e| e.into())
    }

    #[wasm_bindgen(getter, js_name = eventCoordinate)]
    pub fn event_coordinate(&self) -> Option<JsCoordinate> {
        self.inner.event_coordinate.clone().map(|e| e.into())
    }
}
