// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip65;
use nostr::{RelayMetadata, UncheckedUrl};
use wasm_bindgen::prelude::*;

use crate::event::tag::JsRelayMetadata;
use crate::event::JsEvent;

#[wasm_bindgen(js_name = RelayListItem)]
pub struct JsRelayListItem {
    url: String,
    metadata: Option<JsRelayMetadata>,
}

impl From<JsRelayListItem> for (UncheckedUrl, Option<RelayMetadata>) {
    fn from(value: JsRelayListItem) -> Self {
        (
            UncheckedUrl::from(value.url),
            value.metadata.map(|r| r.into()),
        )
    }
}

#[wasm_bindgen(js_class = RelayListItem)]
impl JsRelayListItem {
    pub fn new(url: String, metadata: Option<JsRelayMetadata>) -> Self {
        Self { url, metadata }
    }

    #[wasm_bindgen(getter)]
    pub fn url(&self) -> String {
        self.url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> Option<JsRelayMetadata> {
        self.metadata
    }
}

#[wasm_bindgen(js_name = extractRelayList)]
pub fn extract_relay_list(event: &JsEvent) -> Vec<JsRelayListItem> {
    nip65::extract_relay_list(event.deref())
        .into_iter()
        .map(|(s, r)| JsRelayListItem {
            url: s.to_string(),
            metadata: r.map(|r| r.into()),
        })
        .collect()
}
