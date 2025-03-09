// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Gossip)]
pub struct JsGossip {
    inner: Gossip,
}

impl Deref for JsGossip {
    type Target = Gossip;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = Gossip)]
impl JsGossip {
    #[wasm_bindgen(js_name = inMemory)]
    pub fn in_memory() -> Self {
        Self {
            inner: Gossip::in_memory(),
        }
    }
}
