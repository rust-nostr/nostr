// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::nips::nip46::NostrConnectURI;
use nostr_ffi::{Keys, PublicKey};
use nostr_sdk::{block_on, signer};
use uniffi::Object;

use crate::error::Result;
use crate::relay::RelayOptions;

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
        uri: &NostrConnectURI,
        app_keys: &Keys,
        timeout: Duration,
        opts: Option<Arc<RelayOptions>>,
    ) -> Result<Self> {
        block_on(async move {
            Ok(Self {
                inner: signer::Nip46Signer::new(
                    uri.deref().clone(),
                    app_keys.deref().clone(),
                    timeout,
                    opts.map(|k| k.as_ref().deref().clone()),
                )
                .await?,
            })
        })
    }

    /// Get signer relays
    pub fn relays(&self) -> Vec<String> {
        block_on(async move {
            self.inner
                .relays()
                .await
                .into_iter()
                .map(|u| u.to_string())
                .collect()
        })
    }

    /// Get signer public key
    pub fn signer_public_key(&self) -> PublicKey {
        self.inner.signer_public_key().clone().into()
    }

    /// Get Nostr Connect URI in **bunker** format.
    pub fn nostr_connect_uri(&self) -> NostrConnectURI {
        block_on(async move { self.inner.nostr_connect_uri().await.into() })
    }
}
