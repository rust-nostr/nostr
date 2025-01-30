// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use js_sys::Array;
use nostr_connect::prelude::*;
use wasm_bindgen::prelude::*;

use crate::duration::JsDuration;
use crate::error::{into_err, Result};
use crate::protocol::key::JsKeys;
use crate::protocol::nips::nip46::JsNostrConnectURI;
use crate::JsStringArray;

#[wasm_bindgen(js_name = NostrConnect)]
pub struct JsNostrConnect {
    inner: NostrConnect,
}

impl Deref for JsNostrConnect {
    type Target = NostrConnect;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NostrConnect> for JsNostrConnect {
    fn from(inner: NostrConnect) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NostrConnect)]
impl JsNostrConnect {
    /// Construct Nostr Connect client
    #[wasm_bindgen]
    pub fn new(
        uri: &JsNostrConnectURI,
        app_keys: &JsKeys,
        timeout: &JsDuration,
    ) -> Result<JsNostrConnect> {
        Ok(Self {
            inner: NostrConnect::new(
                uri.deref().clone(),
                app_keys.deref().clone(),
                **timeout,
                None,
            )
            .map_err(into_err)?,
        })
    }

    /// Get signer relays
    #[wasm_bindgen]
    pub fn relays(&self) -> JsStringArray {
        self.inner
            .relays()
            .iter()
            .map(|u| JsValue::from(u.to_string()))
            .collect::<Array>()
            .unchecked_into()
    }

    /// Get `bunker` URI
    #[wasm_bindgen(js_name = bunkerUri)]
    pub async fn bunker_uri(&self) -> Result<JsNostrConnectURI> {
        Ok(self.inner.bunker_uri().await.map_err(into_err)?.into())
    }
}
