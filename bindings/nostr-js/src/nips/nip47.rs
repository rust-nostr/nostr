// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::str::FromStr;

use nostr::nips::nip47::NostrWalletConnectURI;
use nostr::Url;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::key::{JsPublicKey, JsSecretKey};

#[wasm_bindgen(js_name = NostrWalletConnectURI)]
pub struct JsNostrWalletConnectURI {
    inner: NostrWalletConnectURI,
}

#[wasm_bindgen(js_class = NostrWalletConnectURI)]
impl JsNostrWalletConnectURI {
    /// Create new Nostr Wallet Connect URI
    pub fn new(
        public_key: &JsPublicKey,
        relay_url: &str,
        random_secret_key: &JsSecretKey,
        lud16: Option<String>,
    ) -> Result<JsNostrWalletConnectURI> {
        let relay_url = Url::parse(relay_url).map_err(into_err)?;
        Ok(Self {
            inner: NostrWalletConnectURI::new(**public_key, relay_url, **random_secret_key, lud16),
        })
    }

    /// Parse
    pub fn parse(uri: &str) -> Result<JsNostrWalletConnectURI> {
        Ok(Self {
            inner: NostrWalletConnectURI::from_str(uri).map_err(into_err)?,
        })
    }

    /// App Pubkey
    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(&self) -> JsPublicKey {
        self.inner.public_key.into()
    }

    /// URL of the relay of choice where the `App` is connected and the `Signer` must send and listen for messages.
    #[wasm_bindgen(js_name = relayUrl)]
    pub fn relay_url(&self) -> String {
        self.inner.relay_url.to_string()
    }

    /// 32-byte randomly generated hex encoded string
    pub fn secret(&self) -> JsSecretKey {
        self.inner.secret.into()
    }

    /// A lightning address that clients can use to automatically setup the lud16 field on the user's profile if they have none configured.
    pub fn lud16(&self) -> Option<String> {
        self.inner.lud16.clone()
    }

    #[wasm_bindgen(js_name = asString)]
    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }
}
