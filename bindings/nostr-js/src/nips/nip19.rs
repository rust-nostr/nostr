// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::JsEventId;
use crate::key::JsPublicKey;

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
        kind: Option<u16>,
        relays: Vec<String>,
    ) -> Self {
        let mut inner = Nip19Event::new(**event_id, relays);
        inner.author = author.map(|p| *p);
        inner.kind = kind.map(Kind::from);
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

    pub fn kind(&self) -> Option<u16> {
        self.inner.kind.map(|k| k.as_u16())
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

#[wasm_bindgen(js_name = Nip19Relay)]
pub struct JsNip19Relay {
    inner: Nip19Relay,
}

impl From<Nip19Relay> for JsNip19Relay {
    fn from(inner: Nip19Relay) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Nip19Relay)]
impl JsNip19Relay {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<JsNip19Relay> {
        let url: Url = Url::parse(url).map_err(into_err)?;
        Ok(Self {
            inner: Nip19Relay::new(url),
        })
    }

    #[wasm_bindgen(js_name = fromBech32)]
    pub fn from_bech32(bech32: &str) -> Result<JsNip19Relay> {
        Ok(Self {
            inner: Nip19Relay::from_bech32(bech32).map_err(into_err)?,
        })
    }

    #[wasm_bindgen(js_name = fromNostrUri)]
    pub fn from_nostr_uri(uri: &str) -> Result<JsNip19Relay> {
        Ok(Self {
            inner: Nip19Relay::from_nostr_uri(uri).map_err(into_err)?,
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

    #[wasm_bindgen]
    pub fn url(&self) -> String {
        self.inner.url.to_string()
    }
}
