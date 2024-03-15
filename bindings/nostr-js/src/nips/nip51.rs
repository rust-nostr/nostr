// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP51
//!
//! <https://github.com/nostr-protocol/nips/blob/master/51.md>

use nostr::nips::nip51::MuteList;
use wasm_bindgen::prelude::*;

use crate::event::JsEventId;
use crate::key::JsPublicKey;

/// Things the user doesn't want to see in their feeds
#[wasm_bindgen(js_name = MuteList)]
pub struct JsMuteList {
    #[wasm_bindgen(getter_with_clone)]
    pub public_keys: Vec<JsPublicKey>,
    #[wasm_bindgen(getter_with_clone)]
    pub hashtags: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub event_ids: Vec<JsEventId>,
    #[wasm_bindgen(getter_with_clone)]
    pub words: Vec<String>,
}

impl From<JsMuteList> for MuteList {
    fn from(value: JsMuteList) -> Self {
        Self {
            public_keys: value.public_keys.into_iter().map(|p| p.into()).collect(),
            hashtags: value.hashtags,
            event_ids: value.event_ids.into_iter().map(|e| e.into()).collect(),
            words: value.words,
        }
    }
}
