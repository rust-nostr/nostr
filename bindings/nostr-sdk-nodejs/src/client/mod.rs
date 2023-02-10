// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;

use napi::Result;
use nostr_nodejs::key::JsKeys;
use nostr_sdk::prelude::*;

use crate::error::into_err;
use crate::relay::JsRelay;

#[napi(js_name = "Client")]
pub struct JsClient {
    inner: Client,
}

#[napi]
impl JsClient {
    #[napi(constructor)]
    pub fn new(keys: &JsKeys) -> Self {
        Self {
            inner: Client::new(keys.deref()),
        }
    }

    // Add new_with_opts

    /// Update default difficulty for new `Event`
    #[napi]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    // Add update_opts

    /// Get current `Keys`
    #[napi]
    pub fn keys(&self) -> JsKeys {
        self.inner.keys().into()
    }

    /// Completly shutdown `Client`
    #[napi]
    pub async fn shutdown(&self) -> Result<()> {
        self.inner.clone().shutdown().await.map_err(into_err)
    }

    // Add notifications

    /// Get relays
    #[napi]
    pub async fn relays(&self) -> HashMap<String, JsRelay> {
        self.inner
            .relays()
            .await
            .into_iter()
            .map(|(u, r)| (u.to_string(), r.into()))
            .collect()
    }

    /// Add new relay
    #[napi]
    pub async fn add_relay(&self, url: String, proxy: Option<String>) -> Result<()> {
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse().map_err(into_err)?),
            None => None,
        };
        self.inner.add_relay(url, proxy).await.map_err(into_err)
    }
}
