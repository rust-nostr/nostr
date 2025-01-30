// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::protocol::event::{JsEventId, JsKind};
use crate::protocol::key::JsPublicKey;

#[wasm_bindgen(js_name = Nip19Event)]
pub struct JsNip19Event {
    inner: Nip19Event,
}

impl From<Nip19Event> for JsNip19Event {
    fn from(inner: Nip19Event) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Nip19Event)]
impl JsNip19Event {
    #[wasm_bindgen(constructor)]
    pub fn new(
        event_id: &JsEventId,
        author: Option<JsPublicKey>,
        kind: Option<JsKind>,
        relays: Vec<String>,
    ) -> Self {
        let mut inner = Nip19Event::new(**event_id, relays);
        inner.author = author.map(|p| *p);
        inner.kind = kind.map(|k| *k);
        Self { inner }
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(bech32: &str) -> Result<JsNip19Event> {
        Ok(Self {
            inner: Nip19Event::from_bech32(bech32).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromNostrUri)]
    pub fn from_nostr_uri(uri: &str) -> Result<JsNip19Event> {
        Ok(Self {
            inner: Nip19Event::from_nostr_uri(uri).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = toBech32)]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }

    #[wasm_bindgen(js_name = toNostrUri)]
    pub fn to_nostr_uri(&self) -> Result<String> {
        self.inner.to_nostr_uri().map_err(into_err)
    }

    #[wasm_bindgen(js_name = eventId)]
    pub fn event_id(&self) -> JsEventId {
        self.inner.event_id.into()
    }

    pub fn author(&self) -> Option<JsPublicKey> {
        self.inner.author.map(|p| p.into())
    }

    pub fn kind(&self) -> Option<JsKind> {
        self.inner.kind.map(|k| k.into())
    }

    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.clone()
    }
}

#[wasm_bindgen(js_name = Nip19Profile)]
pub struct JsNip19Profile {
    inner: Nip19Profile,
}

impl From<Nip19Profile> for JsNip19Profile {
    fn from(inner: Nip19Profile) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Nip19Profile)]
impl JsNip19Profile {
    /// New NIP19 profile
    #[wasm_bindgen(constructor)]
    pub fn new(public_key: &JsPublicKey, relays: Vec<String>) -> Result<JsNip19Profile> {
        Ok(Self {
            inner: Nip19Profile::new(**public_key, relays).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(bech32: &str) -> Result<JsNip19Profile> {
        Ok(Self {
            inner: Nip19Profile::from_bech32(bech32).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromNostrUri)]
    pub fn from_nostr_uri(uri: &str) -> Result<JsNip19Profile> {
        Ok(Self {
            inner: Nip19Profile::from_nostr_uri(uri).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = toBech32)]
    pub fn to_bech32(&self) -> Result<String> {
        self.inner.to_bech32().map_err(into_err)
    }

    #[wasm_bindgen(js_name = toNostrUri)]
    pub fn to_nostr_uri(&self) -> Result<String> {
        self.inner.to_nostr_uri().map_err(into_err)
    }

    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    pub fn relays(&self) -> Vec<String> {
        self.inner.relays.iter().map(|u| u.to_string()).collect()
    }
}
