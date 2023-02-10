// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::prelude::*;

#[napi(js_name = "Relay")]
pub struct JsRelay {
    inner: Relay,
}

impl From<Relay> for JsRelay {
    fn from(relay: Relay) -> Self {
        Self { inner: relay }
    }
}

#[napi]
impl JsRelay {
    /// Get relay url
    #[napi(getter)]
    pub fn url(&self) -> String {
        self.inner.url().to_string()
    }

    /// Get proxy
    #[napi(getter)]
    pub fn proxy(&self) -> Option<String> {
        self.inner.proxy().map(|p| p.to_string())
    }
}
