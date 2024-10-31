// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::database::DynNostrDatabase;
use nostr_sdk::zapper::DynNostrZapper;
use uniffi::Object;

use super::zapper::NostrZapper;
use super::{Client, ClientSdk, Options};
use crate::database::NostrDatabase;
use crate::protocol::helper::unwrap_or_clone_arc;
use crate::protocol::signer::{NostrSigner, NostrSignerFFI2Rust};

#[derive(Clone, Default, Object)]
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
    #[inline]
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn signer(self: Arc<Self>, signer: Arc<dyn NostrSigner>) -> Self {
        let signer = NostrSignerFFI2Rust::new(signer);
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.signer(signer);
        builder
    }

    pub fn zapper(self: Arc<Self>, zapper: Arc<NostrZapper>) -> Self {
        let zapper: Arc<DynNostrZapper> = zapper.as_ref().deref().clone();
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.zapper(zapper);
        builder
    }

    pub fn database(self: Arc<Self>, database: Arc<NostrDatabase>) -> Self {
        let database: Arc<DynNostrDatabase> = database.as_ref().into();
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.database(database);
        builder
    }

    /// Set opts
    pub fn opts(self: Arc<Self>, opts: Arc<Options>) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.opts(opts.as_ref().deref().clone());
        builder
    }

    /// Build [`Client`]
    pub fn build(&self) -> Arc<Client> {
        let inner = self.inner.clone();
        Arc::new(ClientSdk::from_builder(inner).into())
    }
}
