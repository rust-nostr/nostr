// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP07
//!
//! <https://github.com/nostr-protocol/nips/blob/master/07.md>

use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

use bitcoin::secp256k1;
use bitcoin::secp256k1::schnorr::Signature;
use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Window;

use crate::event::{self, unsigned};
use crate::{key, Event, PublicKey, UnsignedEvent};

/// NIP07 error
#[derive(Debug)]
pub enum Error {
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Keys error
    Keys(key::Error),
    /// Event error
    Event(event::Error),
    /// Unsigned error
    Unsigned(unsigned::Error),
    /// Generic WASM error
    Wasm(JsValue),
    /// Impossible to get window
    NoGlobalWindowObject,
    /// Impossible to get window
    NamespaceNotFound(String),
    /// Object key not found
    ObjectKeyNotFound(String),
    /// Invalid type: expected a string
    TypeMismatch(String),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::Keys(e) => write!(f, "Keys: {e}"),
            Self::Event(e) => write!(f, "Event: {e}"),
            Self::Unsigned(e) => write!(f, "Unsigned event: {e}"),
            Self::Wasm(e) => write!(f, "{e:?}"),
            Self::NoGlobalWindowObject => write!(f, "No global `window` object"),
            Self::NamespaceNotFound(n) => write!(f, "`{n}` namespace not found"),
            Self::ObjectKeyNotFound(n) => write!(f, "Key `{n}` not found in object"),
            Self::TypeMismatch(e) => write!(f, "Type mismatch: {e}"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<unsigned::Error> for Error {
    fn from(e: unsigned::Error) -> Self {
        Self::Unsigned(e)
    }
}

impl From<JsValue> for Error {
    fn from(e: JsValue) -> Self {
        Self::Wasm(e)
    }
}

/// NIP07 Signer for interaction with browser extensions (ex. Alby)
///
/// <https://github.com/aljazceru/awesome-nostr#nip-07-browser-extensions>
#[derive(Debug, Clone)]
pub struct Nip07Signer {
    /// `window.nostr` object
    nostr_obj: Object,
}

impl Nip07Signer {
    /// Compose new NIP07 Signer
    pub fn new() -> Result<Self, Error> {
        let window: Window = web_sys::window().ok_or(Error::NoGlobalWindowObject)?;
        let namespace: JsValue = Reflect::get(&window, &JsValue::from_str("nostr"))
            .map_err(|_| Error::NamespaceNotFound(String::from("nostr")))?;
        let nostr_obj: Object = namespace
            .dyn_into()
            .map_err(|_| Error::NamespaceNotFound(String::from("nostr")))?;
        Ok(Self { nostr_obj })
    }

    /// Check if `window.nostr` object is available
    pub fn is_available() -> bool {
        Self::new().is_ok()
    }

    fn get_func<S>(&self, obj: &Object, name: S) -> Result<Function, Error>
    where
        S: AsRef<str>,
    {
        let name: &str = name.as_ref();
        let val: JsValue = Reflect::get(obj, &JsValue::from_str(name))
            .map_err(|_| Error::NamespaceNotFound(name.to_string()))?;
        val.dyn_into()
            .map_err(|_| Error::NamespaceNotFound(name.to_string()))
    }

    /// Get value from object key
    fn get_value_by_key(&self, obj: &Object, key: &str) -> Result<JsValue, Error> {
        Reflect::get(obj, &JsValue::from_str(key))
            .map_err(|_| Error::ObjectKeyNotFound(key.to_string()))
    }

    /// Get Public Key
    pub async fn get_public_key(&self) -> Result<PublicKey, Error> {
        let func: Function = self.get_func(&self.nostr_obj, "getPublicKey")?;
        let promise: Promise = Promise::resolve(&func.call0(&self.nostr_obj)?);
        let result: JsValue = JsFuture::from(promise).await?;
        let public_key: String = result
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a hex string")))?;
        Ok(PublicKey::from_hex(public_key)?)
    }

    /// Sign event
    pub async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Error> {
        let func: Function = self.get_func(&self.nostr_obj, "signEvent")?;

        let tags: Array = unsigned
            .tags
            .iter()
            .map(|t| {
                t.as_vec()
                    .into_iter()
                    .map(|v| JsValue::from_str(&v))
                    .collect::<Array>()
            })
            .collect();

        let unsigned_obj = Object::new();
        Reflect::set(
            &unsigned_obj,
            &JsValue::from_str("id"),
            &unsigned.id.to_hex().into(),
        )?;
        Reflect::set(
            &unsigned_obj,
            &JsValue::from_str("pubkey"),
            &unsigned.pubkey.to_string().into(),
        )?;
        Reflect::set(
            &unsigned_obj,
            &JsValue::from_str("created_at"),
            &(unsigned.created_at.as_u64() as f64).into(),
        )?;
        Reflect::set(
            &unsigned_obj,
            &JsValue::from_str("kind"),
            &(unsigned.kind.as_u64() as f64).into(),
        )?;
        Reflect::set(&unsigned_obj, &JsValue::from_str("tags"), &tags.into())?;
        Reflect::set(
            &unsigned_obj,
            &JsValue::from_str("content"),
            &unsigned.content.as_str().into(),
        )?;

        let promise: Promise = Promise::resolve(&func.call1(&self.nostr_obj, &unsigned_obj)?);
        let result: JsValue = JsFuture::from(promise).await?;
        let event_obj: Object = result.dyn_into()?;

        // Extract signature from event object
        let sig: String = self
            .get_value_by_key(&event_obj, "sig")?
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a hex string")))?;
        let sig: Signature = Signature::from_str(&sig)?;

        // Add signature to unsigned event
        let event: Event = unsigned.add_signature(sig)?;

        // Verify event (both ID and signature)
        event.verify()?;

        Ok(event)
    }

    // TODO: add `signSchnorr`

    // TODO: add `getRelays`

    fn nip04_obj(&self) -> Result<Object, Error> {
        let namespace: JsValue = Reflect::get(&self.nostr_obj, &JsValue::from_str("nip04"))
            .map_err(|_| Error::NamespaceNotFound(String::from("nip04")))?;
        namespace
            .dyn_into()
            .map_err(|_| Error::NamespaceNotFound(String::from("nip04")))
    }

    /// NIP04 encrypt
    pub async fn nip04_encrypt<S>(
        &self,
        public_key: PublicKey,
        plaintext: S,
    ) -> Result<String, Error>
    where
        S: AsRef<str>,
    {
        let nip04_obj: Object = self.nip04_obj()?;
        let func: Function = self.get_func(&nip04_obj, "encrypt")?;
        let promise: Promise = Promise::resolve(&func.call2(
            &nip04_obj,
            &JsValue::from_str(&public_key.to_string()),
            &JsValue::from_str(plaintext.as_ref()),
        )?);
        let result: JsValue = JsFuture::from(promise).await?;
        result
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a string")))
    }

    /// NIP04 decrypt
    pub async fn nip04_decrypt<S>(
        &self,
        public_key: PublicKey,
        ciphertext: S,
    ) -> Result<String, Error>
    where
        S: AsRef<str>,
    {
        let nip04_obj: Object = self.nip04_obj()?;
        let func: Function = self.get_func(&nip04_obj, "decrypt")?;
        let promise: Promise = Promise::resolve(&func.call2(
            &nip04_obj,
            &JsValue::from_str(&public_key.to_string()),
            &JsValue::from_str(ciphertext.as_ref()),
        )?);
        let result: JsValue = JsFuture::from(promise).await?;
        result
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a string")))
    }
}
