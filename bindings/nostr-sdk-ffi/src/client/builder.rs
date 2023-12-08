// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developersopers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::helper::unwrap_or_clone_arc;
use nostr_ffi::Keys;
use nostr_sdk::database::DynNostrDatabase;
use uniffi::Object;

use super::{Client, ClientSdk, Options};
use crate::database::NostrDatabase;

#[derive(Clone, Object)]
pub struct ClientBuilder {
    inner: nostr_sdk::ClientBuilder,
}

impl From<nostr_sdk::ClientBuilder> for ClientBuilder {
    fn from(inner: nostr_sdk::ClientBuilder) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl ClientBuilder {
    /// New client builder
    #[uniffi::constructor]
    pub fn new(keys: Arc<Keys>) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr_sdk::ClientBuilder::new(keys.as_ref().deref()),
        })
    }

    pub fn database(self: Arc<Self>, database: Arc<NostrDatabase>) -> Arc<Self> {
        let database: Arc<DynNostrDatabase> = database.as_ref().into();
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.database(database);
        Arc::new(builder)
    }

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
