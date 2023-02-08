// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;

use napi::bindgen_prelude::*;
use napi::Result;
use nostr_nodejs::key::JsKeys;
use nostr_sdk::Client;

use crate::error::into_err;

#[napi(js_name = "Client")]
pub struct JsClient {
    client: Client,
}

#[napi]
impl JsClient {
    #[napi(constructor)]
    pub fn new(keys: External<JsKeys>) -> Self {
        Self {
            client: Client::new(keys.deref()),
        }
    }

    #[napi]
    pub async fn add_relay(&self, url: String, proxy: Option<String>) -> Result<()> {
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse().map_err(into_err)?),
            None => None,
        };
        self.client.add_relay(url, proxy).await.map_err(into_err)
    }
}
