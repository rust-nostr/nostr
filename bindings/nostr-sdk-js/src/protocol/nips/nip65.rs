// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::protocol::event::JsEvent;

#[derive(Clone, Copy)]
#[wasm_bindgen(js_name = RelayMetadata)]
pub enum JsRelayMetadata {
    Read,
    Write,
}

impl From<RelayMetadata> for JsRelayMetadata {
    fn from(value: RelayMetadata) -> Self {
        match value {
            RelayMetadata::Read => Self::Read,
            RelayMetadata::Write => Self::Write,
        }
    }
}

impl From<JsRelayMetadata> for RelayMetadata {
    fn from(value: JsRelayMetadata) -> Self {
        match value {
            JsRelayMetadata::Read => Self::Read,
            JsRelayMetadata::Write => Self::Write,
        }
    }
}

#[wasm_bindgen(js_name = RelayListItem)]
pub struct JsRelayListItem {
    #[wasm_bindgen(getter_with_clone)]
    pub url: String,
    pub metadata: Option<JsRelayMetadata>,
}

#[wasm_bindgen(js_class = RelayListItem)]
impl JsRelayListItem {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, metadata: Option<JsRelayMetadata>) -> Self {
        Self { url, metadata }
    }
}

#[wasm_bindgen(js_name = extractRelayList)]
pub fn extract_relay_list(event: &JsEvent) -> Vec<JsRelayListItem> {
    nip65::extract_relay_list(event.deref())
        .map(|(s, r)| JsRelayListItem {
            url: s.to_string(),
            metadata: r.map(|r| r.into()),
        })
        .collect()
}
