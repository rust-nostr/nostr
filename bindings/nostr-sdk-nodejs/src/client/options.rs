// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use nostr_sdk::Options;

#[napi(js_name = "Options")]
pub struct JsOptions {
    inner: Options,
}

impl From<&JsOptions> for Options {
    fn from(options: &JsOptions) -> Self {
        options.inner.clone()
    }
}

#[napi]
impl JsOptions {
    /// Create new (default) `Options`
    #[allow(clippy::new_without_default)]
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Options::new(),
        }
    }

    /// If set to `true`, `Client` wait that `Relay` try at least one time to enstablish a connection before continue.
    #[napi]
    pub fn wait_for_connection(&self, wait: bool) -> Self {
        Self {
            inner: self.inner.to_owned().wait_for_connection(wait),
        }
    }

    /// If set to `true`, [`Client`] wait that an event is sent before continue.
    #[napi]
    pub fn wait_for_send(&self, wait: bool) -> Self {
        Self {
            inner: self.inner.to_owned().wait_for_send(wait),
        }
    }

    /// Set default POW diffficulty for `Event`
    #[napi]
    pub fn difficulty(&self, difficulty: u8) -> Self {
        Self {
            inner: self.inner.to_owned().difficulty(difficulty),
        }
    }
}
