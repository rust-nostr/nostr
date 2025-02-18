// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::options::JsOptions;
use super::{JsClient, JsNostrSigner};
use crate::database::JsNostrDatabase;
use crate::policy::{FFI2RustAdmitPolicy, JsAdmitPolicy};

#[wasm_bindgen(js_name = ClientBuilder)]
pub struct JsClientBuilder {
    inner: ClientBuilder,
}

impl From<ClientBuilder> for JsClientBuilder {
    fn from(inner: ClientBuilder) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = ClientBuilder)]
impl JsClientBuilder {
    /// New client builder
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ClientBuilder::new(),
        }
    }

    pub fn signer(self, signer: &JsNostrSigner) -> Self {
        self.inner.signer(signer.deref().clone()).into()
    }

    pub fn database(self, database: &JsNostrDatabase) -> Self {
        self.inner.database(database.deref().clone()).into()
    }

    #[wasm_bindgen(js_name = admitPolicy)]
    pub fn admit_policy(self, policy: JsAdmitPolicy) -> Self {
        self.inner
            .admit_policy(FFI2RustAdmitPolicy { inner: policy })
            .into()
    }

    pub fn opts(self, opts: &JsOptions) -> Self {
        self.inner.opts(opts.deref().clone()).into()
    }

    /// Build `Client`
    ///
    /// This method **consumes** the `ClientBuilder`!
    pub fn build(self) -> JsClient {
        self.inner.build().into()
    }
}
