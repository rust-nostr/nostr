// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developersopers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::database::DynNostrDatabase;
use nostr_sdk::{Client, ClientBuilder};
use wasm_bindgen::prelude::*;

use super::options::JsOptions;
use super::zapper::JsNostrZapper;
use super::{JsClient, JsNostrSigner};
use crate::database::JsNostrDatabase;

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

    pub fn zapper(self, zapper: &JsNostrZapper) -> Self {
        self.inner.zapper(zapper.deref().clone()).into()
    }

    pub fn database(self, database: &JsNostrDatabase) -> Self {
        let database: Arc<DynNostrDatabase> = database.into();
        self.inner.database(database).into()
    }

    pub fn opts(self, opts: &JsOptions) -> Self {
        self.inner.opts(opts.deref().clone()).into()
    }

    /// Build `Client`
    ///
    /// This method **consume** the `ClientBuilder`!
    pub fn build(mut self) -> JsClient {
        self.inner.opts = self.inner.opts.shutdown_on_drop(true);
        Client::from_builder(self.inner).into()
    }
}
