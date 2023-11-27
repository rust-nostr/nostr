// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::helper::unwrap_or_clone_arc;
use nostr_ffi::Keys;

use super::{Client, ClientSdk, Options};

#[derive(Clone)]
pub struct ClientBuilder {
    inner: nostr_sdk::ClientBuilder,
}

impl From<nostr_sdk::ClientBuilder> for ClientBuilder {
    fn from(inner: nostr_sdk::ClientBuilder) -> Self {
        Self { inner }
    }
}

impl ClientBuilder {
    /// New client builder
    pub fn new(keys: Arc<Keys>) -> Self {
        Self {
            inner: nostr_sdk::ClientBuilder::new(keys.as_ref().deref()),
        }
    }

    // TODO: add `database`

    /// Set opts
    pub fn opts(self: Arc<Self>, opts: Arc<Options>) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder
            .inner
            .opts(opts.as_ref().deref().clone().shutdown_on_drop(true));
        Arc::new(builder)
    }

    /// Build [`Client`]
    pub fn build(&self) -> Arc<Client> {
        Arc::new(ClientSdk::from_builder(self.inner.clone()).into())
    }
}
