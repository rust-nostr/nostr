// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP07: `window.nostr` capability for web browsers
//!
//! <https://github.com/nostr-protocol/nips/blob/master/07.md>

use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

use js_sys::{Array, Function, JsString, Object, Promise, Reflect};
use secp256k1::schnorr::Signature;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Window;

use crate::signer::{NostrSigner, SignerBackend, SignerError};
use crate::util::BoxedFuture;
use crate::{event, key, Event, PublicKey, UnsignedEvent};

const GET_PUBLIC_KEY: &str = "getPublicKey";
const SIGN_EVENT: &str = "signEvent";
const NIP04: &str = "nip04";
const NIP44: &str = "nip44";
const ENCRYPT: &str = "encrypt";
const DECRYPT: &str = "decrypt";

enum CallFunc<'a> {
    Call0,
    Call1(&'a JsValue),
    Call2(&'a JsValue, &'a JsValue),
}

/// NIP07 error
#[derive(Debug)]
pub enum Error {
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Keys error
    Keys(key::Error),
    /// Unsigned error
    Event(event::Error),
    /// Generic WASM error
    Wasm(String),
    /// Impossible to get window
    NoGlobalWindowObject,
    /// Namespace not found
    NamespaceNotFound(String),
    /// Object key not found
    ObjectKeyNotFound(String),
    /// Invalid type
    TypeMismatch,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::Keys(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::Wasm(e) => write!(f, "{e}"),
            Self::NoGlobalWindowObject => write!(f, "No global `window` object"),
            Self::NamespaceNotFound(n) => write!(f, "`{n}` namespace not found"),
            Self::ObjectKeyNotFound(n) => write!(f, "Key `{n}` not found in object"),
            Self::TypeMismatch => write!(f, "Type mismatch"),
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

impl From<JsValue> for Error {
    fn from(e: JsValue) -> Self {
        Self::Wasm(format!("{e:?}"))
    }
}

#[allow(missing_docs)]
#[deprecated(since = "0.39.0", note = "BrowserSigner")]
pub type Nip07Signer = BrowserSigner;

/// Signer for interaction with browser extensions (ex. Alby)
///
/// Browser extensions: <https://github.com/aljazceru/awesome-nostr#nip-07-browser-extensions>
///
/// <https://github.com/nostr-protocol/nips/blob/master/07.md>
#[derive(Debug, Clone)]
pub struct BrowserSigner {
    /// `window.nostr` object
    nostr_obj: Object,
}

#[allow(unsafe_code)]
unsafe impl Send for BrowserSigner {}

#[allow(unsafe_code)]
unsafe impl Sync for BrowserSigner {}

impl BrowserSigner {
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

    fn get_func(&self, obj: &Object, name: &str) -> Result<Function, Error> {
        let val: JsValue = Reflect::get(obj, &JsValue::from_str(name))
            .map_err(|_| Error::NamespaceNotFound(name.to_string()))?;
        val.dyn_into()
            .map_err(|_| Error::NamespaceNotFound(name.to_string()))
    }

    fn get_sub_obj(&self, super_obj: &Object, name: &str) -> Result<Object, Error> {
        let namespace: JsValue = Reflect::get(super_obj, &JsValue::from_str(name))
            .map_err(|_| Error::NamespaceNotFound(String::from(name)))?;
        namespace
            .dyn_into()
            .map_err(|_| Error::NamespaceNotFound(String::from(name)))
    }

    /// Get value from object key
    #[inline]
    fn get_value_by_key(&self, obj: &Object, key: &str) -> Result<JsValue, Error> {
        Reflect::get(obj, &JsValue::from_str(key))
            .map_err(|_| Error::ObjectKeyNotFound(key.to_string()))
    }

    async fn call_func<T>(&self, obj: &Object, name: &str, args: CallFunc<'_>) -> Result<T, Error>
    where
        T: JsCast,
    {
        let func: Function = self.get_func(obj, name)?;
        let temp: JsValue = match args {
            CallFunc::Call0 => func.call0(obj)?,
            CallFunc::Call1(arg) => func.call1(obj, arg)?,
            CallFunc::Call2(arg1, arg2) => func.call2(obj, arg1, arg2)?,
        };
        let promise: Promise = Promise::resolve(&temp);
        let result: JsValue = JsFuture::from(promise).await?;

        result.dyn_into().map_err(|_| Error::TypeMismatch)
    }

    /// Get Public Key
    async fn _get_public_key(&self) -> Result<PublicKey, Error> {
        let public_key: JsString = self
            .call_func(&self.nostr_obj, GET_PUBLIC_KEY, CallFunc::Call0)
            .await?;
        let public_key: String = public_key.into();
        Ok(PublicKey::from_hex(&public_key)?)
    }

    async fn _sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, Error> {
        let unsigned_obj = Object::new();

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

        if let Some(id) = unsigned.id {
            Reflect::set(&unsigned_obj, &JsValue::from_str("id"), &id.to_hex().into())?;
        }

        Reflect::set(
            &unsigned_obj,
            &JsValue::from_str("pubkey"),
            &unsigned.pubkey.to_hex().into(),
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

        let event_obj: Object = self
            .call_func(&self.nostr_obj, SIGN_EVENT, CallFunc::Call1(&unsigned_obj))
            .await?;

        // Extract signature from event object
        let sig: String = self
            .get_value_by_key(&event_obj, "sig")?
            .as_string()
            .ok_or(Error::TypeMismatch)?;
        let sig: Signature = Signature::from_str(&sig)?;

        // Add signature
        Ok(unsigned.add_signature(sig)?)
    }

    // TODO: add `getRelays`

    async fn encryption_decryption(
        &self,
        namespace: &str,
        func_name: &str,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, Error> {
        let sub_obj: Object = self.get_sub_obj(&self.nostr_obj, namespace)?;
        let pk: JsValue = JsValue::from_str(&public_key.to_hex());
        let content: JsValue = JsValue::from_str(content);
        let result: JsString = self
            .call_func(&sub_obj, func_name, CallFunc::Call2(&pk, &content))
            .await?;
        Ok(result.into())
    }
}

impl NostrSigner for BrowserSigner {
    fn backend(&self) -> SignerBackend {
        SignerBackend::BrowserExtension
    }

    fn get_public_key(&self) -> BoxedFuture<Result<PublicKey, SignerError>> {
        Box::pin(async move { self._get_public_key().await.map_err(SignerError::backend) })
    }

    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<Result<Event, SignerError>> {
        Box::pin(async move {
            self._sign_event(unsigned)
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip04_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self.encryption_decryption(NIP04, ENCRYPT, public_key, content)
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip04_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self.encryption_decryption(NIP04, DECRYPT, public_key, content)
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self.encryption_decryption(NIP44, ENCRYPT, public_key, content)
                .await
                .map_err(SignerError::backend)
        })
    }

    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self.encryption_decryption(NIP44, DECRYPT, public_key, content)
                .await
                .map_err(SignerError::backend)
        })
    }
}
