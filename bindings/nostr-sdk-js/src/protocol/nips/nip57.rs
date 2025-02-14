// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::nip01::JsCoordinate;
use crate::error::{into_err, Result};
use crate::protocol::event::{JsEvent, JsEventId};
use crate::protocol::key::{JsKeys, JsPublicKey, JsSecretKey};

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
    #[wasm_bindgen(constructor)]
    pub fn new(
        public_key: &JsPublicKey,
        relays: Vec<String>,
        message: &str,
        amount: Option<f64>,
        lnurl: Option<String>,
        event_id: Option<JsEventId>,
        event_coordinate: Option<JsCoordinate>,
    ) -> Self {
        Self {
            inner: ZapRequestData {
                public_key: **public_key,
                relays: relays
                    .into_iter()
                    .filter_map(|r| RelayUrl::parse(&r).ok())
                    .collect(),
                message: message.to_string(),
                amount: amount.map(|n| n as u64),
                lnurl,
                event_id: event_id.map(|e| *e.deref()),
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

    #[wasm_bindgen(getter, js_name = eventId)]
    pub fn event_id(&self) -> Option<JsEventId> {
        self.inner.event_id.map(|e| e.into())
    }

    #[wasm_bindgen(getter, js_name = eventCoordinate)]
    pub fn event_coordinate(&self) -> Option<JsCoordinate> {
        self.inner.event_coordinate.clone().map(|e| e.into())
    }
}

#[wasm_bindgen(js_name = nip57AnonymousZapRequest)]
pub fn nip57_anonymous_zap_request(data: &JsZapRequestData) -> Result<JsEvent> {
    Ok(nip57::anonymous_zap_request(data.deref().clone())
        .map_err(into_err)?
        .into())
}

#[wasm_bindgen(js_name = nip57PrivateZapRequest)]
pub fn nip57_private_zap_request(data: &JsZapRequestData, keys: &JsKeys) -> Result<JsEvent> {
    Ok(
        nip57::private_zap_request(data.deref().clone(), keys.deref())
            .map_err(into_err)?
            .into(),
    )
}

#[wasm_bindgen(js_name = nip57DecryptSentPrivateZapMessage)]
pub fn decrypt_sent_private_zap_message(
    secret_key: &JsSecretKey,
    public_key: &JsPublicKey,
    private_zap: &JsEvent,
) -> Result<JsEvent> {
    Ok(nip57::decrypt_sent_private_zap_message(
        secret_key.deref(),
        public_key.deref(),
        private_zap.deref(),
    )
    .map_err(into_err)?
    .into())
}

#[wasm_bindgen(js_name = nip57DecryptReceivedPrivateZapMessage)]
pub fn decrypt_received_private_zap_message(
    secret_key: &JsSecretKey,
    private_zap: &JsEvent,
) -> Result<JsEvent> {
    Ok(
        nip57::decrypt_received_private_zap_message(secret_key.deref(), private_zap.deref())
            .map_err(into_err)?
            .into(),
    )
}
