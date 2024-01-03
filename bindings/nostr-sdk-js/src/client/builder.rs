// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developersopers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::ClientBuilder;
use nostr_sdk::{database::DynNostrDatabase, Client};
use wasm_bindgen::prelude::*;

use super::{JsClient, JsClientSigner};
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

    pub fn signer(self, signer: &JsClientSigner) -> Self {
        self.inner.signer(signer.deref().clone()).into()
    }

    pub fn database(self, database: &JsNostrDatabase) -> Self {
        let database: Arc<DynNostrDatabase> = database.into();
        self.inner.database(database).into()
    }

    // TODO: add `opts`

    /// Build [`Client`]
    pub fn build(&self) -> JsClient {
        Client::from_builder(self.inner.clone()).into()
    }
}
