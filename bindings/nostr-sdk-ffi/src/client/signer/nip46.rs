// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::{Keys, PublicKey};
use nostr_sdk::{block_on, client, Url};
use uniffi::Object;

use crate::error::Result;

#[derive(Object)]
pub struct Nip46Signer {
    inner: client::Nip46Signer,
}

impl Deref for Nip46Signer {
    type Target = client::Nip46Signer;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<client::Nip46Signer> for Nip46Signer {
    fn from(inner: client::Nip46Signer) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Nip46Signer {
    /// New NIP46 remote signer
    #[uniffi::constructor]
    pub fn new(
        relay_url: String,
        app_keys: Arc<Keys>,
        signer_public_key: Option<Arc<PublicKey>>,
    ) -> Result<Self> {
        let relay_url: Url = Url::parse(&relay_url)?;
        Ok(Self {
            inner: client::Nip46Signer::new(
                relay_url,
                app_keys.as_ref().deref().clone(),
                signer_public_key.map(|p| **p),
            ),
        })
    }

    /// Get signer relay [`Url`]
    pub fn relay_url(&self) -> String {
        self.inner.relay_url().to_string()
    }

    /// Get signer [`XOnlyPublicKey`]
    pub fn signer_public_key(&self) -> Option<Arc<PublicKey>> {
        block_on(async move {
            self.inner
                .signer_public_key()
                .await
                .map(|p| Arc::new(p.into()))
        })
    }

    // TODO: add nostr_connect_uri
}
