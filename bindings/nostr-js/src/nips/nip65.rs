// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip65;
use wasm_bindgen::prelude::*;

use crate::event::tag::JsRelayMetadata;
use crate::event::JsEvent;

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
        .into_iter()
        .map(|(s, r)| JsRelayListItem {
            url: s.to_string(),
            metadata: r.clone().map(|r| r.into()),
        })
        .collect()
}
