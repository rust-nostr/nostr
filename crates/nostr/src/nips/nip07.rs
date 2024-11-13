// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP07: `window.nostr` capability for web browsers
//!
//! <https://github.com/nostr-protocol/nips/blob/master/07.md>

use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

use async_trait::async_trait;
use bitcoin::secp256k1;
use bitcoin::secp256k1::schnorr::Signature;
use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Window;

use crate::event::{self, unsigned};
use crate::signer::{NostrSigner, SignerBackend, SignerError};
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
    Wasm(String),
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
            Self::Wasm(e) => write!(f, "{e}"),
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
    #[inline]
    fn from(e: JsValue) -> Self {
        Self::Wasm(format!("{e:?}"))
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
    #[inline]
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
    #[inline]
    fn get_value_by_key(&self, obj: &Object, key: &str) -> Result<JsValue, Error> {
        Reflect::get(obj, &JsValue::from_str(key))
            .map_err(|_| Error::ObjectKeyNotFound(key.to_string()))
    }

    /// Get Public Key
    async fn _get_public_key(&self) -> Result<PublicKey, Error> {
        let func: Function = self.get_func(&self.nostr_obj, "getPublicKey")?;
        let promise: Promise = Promise::resolve(&func.call0(&self.nostr_obj)?);
        let result: JsValue = JsFuture::from(promise).await?;
        let public_key: String = result
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a hex string")))?;
        Ok(PublicKey::from_hex(public_key)?)
    }

    async fn _sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Error> {
        let func: Function = self.get_func(&self.nostr_obj, "signEvent")?;

        let tags: Array = unsigned
            .tags
            .iter()
            .map(|t| {
                t.as_slice()
                    .iter()
                    .map(|v| JsValue::from_str(v))
                    .collect::<Array>()
            })
            .collect();

        let unsigned_obj = Object::new();

        if let Some(id) = unsigned.id {
            Reflect::set(&unsigned_obj, &JsValue::from_str("id"), &id.to_hex().into())?;
        }

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
            &(unsigned.kind.as_u16() as f64).into(),
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

        // Add signature
        Ok(unsigned.add_signature(sig)?)
    }

    // TODO: add `getRelays`

    fn nip04_obj(&self) -> Result<Object, Error> {
        let namespace: JsValue = Reflect::get(&self.nostr_obj, &JsValue::from_str("nip04"))
            .map_err(|_| Error::NamespaceNotFound(String::from("nip04")))?;
        namespace
            .dyn_into()
            .map_err(|_| Error::NamespaceNotFound(String::from("nip04")))
    }

    async fn _nip04_encrypt<T>(&self, public_key: &PublicKey, content: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let nip04_obj: Object = self.nip04_obj()?;
        let func: Function = self.get_func(&nip04_obj, "encrypt")?;
        let content: &[u8] = content.as_ref();
        let content: String = String::from_utf8_lossy(content).to_string();
        let promise: Promise = Promise::resolve(&func.call2(
            &nip04_obj,
            &JsValue::from_str(&public_key.to_hex()),
            &JsValue::from_str(&content),
        )?);
        let result: JsValue = JsFuture::from(promise).await?;
        result
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a string")))
    }

    async fn _nip04_decrypt<S>(
        &self,
        public_key: &PublicKey,
        ciphertext: S,
    ) -> Result<String, Error>
    where
        S: AsRef<str>,
    {
        let nip04_obj: Object = self.nip04_obj()?;
        let func: Function = self.get_func(&nip04_obj, "decrypt")?;
        let promise: Promise = Promise::resolve(&func.call2(
            &nip04_obj,
            &JsValue::from_str(&public_key.to_hex()),
            &JsValue::from_str(ciphertext.as_ref()),
        )?);
        let result: JsValue = JsFuture::from(promise).await?;
        result
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a string")))
    }

    fn nip44_obj(&self) -> Result<Object, Error> {
        let namespace: JsValue = Reflect::get(&self.nostr_obj, &JsValue::from_str("nip44"))
            .map_err(|_| Error::NamespaceNotFound(String::from("nip44")))?;
        namespace
            .dyn_into()
            .map_err(|_| Error::NamespaceNotFound(String::from("nip44")))
    }

    async fn _nip44_encrypt<T>(&self, public_key: &PublicKey, content: T) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let nip44_obj: Object = self.nip44_obj()?;
        let func: Function = self.get_func(&nip44_obj, "encrypt")?;
        let content: &[u8] = content.as_ref();
        let content: String = String::from_utf8_lossy(content).to_string();
        let promise: Promise = Promise::resolve(&func.call2(
            &nip44_obj,
            &JsValue::from_str(&public_key.to_hex()),
            &JsValue::from_str(&content),
        )?);
        let result: JsValue = JsFuture::from(promise).await?;
        result
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a string")))
    }

    async fn _nip44_decrypt<T>(
        &self,
        public_key: &PublicKey,
        ciphertext: T,
    ) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        let nip44_obj: Object = self.nip44_obj()?;
        let func: Function = self.get_func(&nip44_obj, "decrypt")?;
        let ciphertext: &[u8] = ciphertext.as_ref();
        let ciphertext: String = String::from_utf8_lossy(ciphertext).to_string();
        let promise: Promise = Promise::resolve(&func.call2(
            &nip44_obj,
            &JsValue::from_str(&public_key.to_hex()),
            &JsValue::from_str(&ciphertext),
        )?);
        let result: JsValue = JsFuture::from(promise).await?;
        result
            .as_string()
            .ok_or_else(|| Error::TypeMismatch(String::from("expected a string")))
    }
}

#[async_trait(?Send)]
impl NostrSigner for Nip07Signer {
    fn backend(&self) -> SignerBackend {
        SignerBackend::BrowserExtension
    }

    async fn get_public_key(&self) -> Result<PublicKey, SignerError> {
        self._get_public_key().await.map_err(SignerError::backend)
    }

    async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, SignerError> {
        self._sign_event(unsigned)
            .await
            .map_err(SignerError::backend)
    }

    async fn nip04_encrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self._nip04_encrypt(public_key, content)
            .await
            .map_err(SignerError::backend)
    }

    async fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self._nip04_decrypt(public_key, content)
            .await
            .map_err(SignerError::backend)
    }

    async fn nip44_encrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self._nip44_encrypt(public_key, content)
            .await
            .map_err(SignerError::backend)
    }

    async fn nip44_decrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self._nip44_decrypt(public_key, content)
            .await
            .map_err(SignerError::backend)
    }
}
