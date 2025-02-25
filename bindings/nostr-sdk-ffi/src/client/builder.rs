// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use uniffi::Object;

use super::{Client, Options};
use crate::database::NostrDatabase;
use crate::policy::{AdmitPolicy, FFI2RustAdmitPolicy};
use crate::protocol::signer::NostrSigner;
use crate::transport::websocket::{CustomWebSocketTransport, FFI2RustWebSocketTransport};

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
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn signer(&self, signer: &NostrSigner) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.signer(signer.deref().clone());
        builder
    }

    pub fn database(&self, database: &NostrDatabase) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.database(database.deref().clone());
        builder
    }

    /// Set a custom WebSocket transport
    pub fn websocket_transport(&self, transport: Arc<dyn CustomWebSocketTransport>) -> Self {
        let mut builder = self.clone();
        let intermediate = FFI2RustWebSocketTransport { inner: transport };
        builder.inner = builder.inner.websocket_transport(intermediate);
        builder
    }

    /// Set an admission policy
    pub fn admit_policy(&self, policy: Arc<dyn AdmitPolicy>) -> Self {
        let mut builder = self.clone();
        let intermediate = FFI2RustAdmitPolicy { inner: policy };
        builder.inner = builder.inner.admit_policy(intermediate);
        builder
    }

    /// Set opts
    pub fn opts(&self, opts: &Options) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.opts(opts.deref().clone());
        builder
    }

    /// Build [`Client`]
    pub fn build(&self) -> Client {
        let inner = self.inner.clone();
        inner.build().into()
    }
}
