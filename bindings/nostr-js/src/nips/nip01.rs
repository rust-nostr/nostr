// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip01::Coordinate;
use nostr::Kind;
use wasm_bindgen::prelude::*;

use crate::key::JsPublicKey;

#[wasm_bindgen(js_name = Coordinate)]
#[derive(Clone)]
pub struct JsCoordinate {
    inner: Coordinate,
}

impl Deref for JsCoordinate {
    type Target = Coordinate;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Coordinate> for JsCoordinate {
    fn from(inner: Coordinate) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Coordinate)]
impl JsCoordinate {
    #[wasm_bindgen(constructor)]
    pub fn new(
        kind: u16,
        public_key: &JsPublicKey,
        identifier: Option<String>,
        relays: Option<Vec<String>>,
    ) -> Self {
        Self {
            inner: Coordinate {
                kind: Kind::from(kind),
                public_key: **public_key,
                identifier: identifier.unwrap_or_default(),
                relays: relays.unwrap_or_default(),
            },
        }
    }

    #[inline]
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> u16 {
        self.inner.kind.as_u16()
    }

    #[wasm_bindgen(getter, js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    #[wasm_bindgen(getter)]
    pub fn identifier(&self) -> String {
        self.inner.identifier.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.clone()
    }
}
