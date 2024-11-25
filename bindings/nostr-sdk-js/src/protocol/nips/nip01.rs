// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::event::JsKind;
use crate::protocol::key::JsPublicKey;

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
        kind: &JsKind,
        public_key: &JsPublicKey,
        identifier: Option<String>,
        relays: Option<Vec<String>>,
    ) -> Self {
        Self {
            inner: Coordinate {
                kind: **kind,
                public_key: **public_key,
                identifier: identifier.unwrap_or_default(),
                // TODO: propagate error
                relays: relays
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|u| RelayUrl::parse(u).ok())
                    .collect(),
            },
        }
    }

    /// Parse coordinate from `<kind>:<pubkey>:[<d-tag>]` format, `bech32` or [NIP21](https://github.com/nostr-protocol/nips/blob/master/21.md) uri
    #[wasm_bindgen]
    pub fn parse(coordinate: &str) -> Result<JsCoordinate> {
        Ok(Self {
            inner: Coordinate::parse(coordinate).map_err(into_err)?,
        })
    }

    #[inline]
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> JsKind {
        self.inner.kind.into()
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
        self.inner.relays.iter().map(|u| u.to_string()).collect()
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn _to_string(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen(js_name = toBech32)]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }

    #[wasm_bindgen(js_name = toNostrUri)]
    pub fn to_nostr_uri(&self) -> Result<String> {
        self.inner.to_nostr_uri().map_err(into_err)
    }
}
