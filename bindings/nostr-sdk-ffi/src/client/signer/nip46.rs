// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::nips::nip46::{NostrConnectMetadata, NostrConnectURI};
use nostr_ffi::{Keys, PublicKey};
use nostr_sdk::{block_on, signer, Url};
use uniffi::Object;

use crate::error::Result;

#[derive(Object)]
pub struct Nip46Signer {
    inner: signer::Nip46Signer,
}

impl Deref for Nip46Signer {
    type Target = signer::Nip46Signer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<signer::Nip46Signer> for Nip46Signer {
    fn from(inner: signer::Nip46Signer) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Nip46Signer {
    /// New NIP46 remote signer
    #[uniffi::constructor]
    pub fn new(
        relay_url: String,
        app_keys: &Keys,
        signer_public_key: Option<Arc<PublicKey>>,
        timeout: Duration,
    ) -> Result<Self> {
        block_on(async move {
            let relay_url: Url = Url::parse(&relay_url)?;
            Ok(Self {
                inner: signer::Nip46Signer::new(
                    relay_url,
                    app_keys.deref().clone(),
                    signer_public_key.map(|p| **p),
                    timeout,
                )
                .await?,
            })
        })
    }

    /// Get signer relay [`Url`]
    pub fn relay_url(&self) -> String {
        self.inner.relay_url().to_string()
    }

    /// Get signer public key
    pub fn signer_public_key(&self) -> Result<PublicKey> {
        block_on(async move { Ok(self.inner.signer_public_key().await?.into()) })
    }

    pub fn nostr_connect_uri(&self, metadata: &NostrConnectMetadata) -> NostrConnectURI {
        self.inner
            .nostr_connect_uri(metadata.deref().clone())
            .into()
    }
}
