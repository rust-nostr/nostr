// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr::NostrSigner;
use nostr_connect::client;
use uniffi::Object;

use crate::error::Result;
use crate::protocol::event::{Event, UnsignedEvent};
use crate::protocol::key::{Keys, PublicKey};
use crate::protocol::nips::nip46::NostrConnectURI;
use crate::relay::RelayOptions;

#[derive(Object)]
pub struct NostrConnect {
    inner: client::NostrConnect,
}

impl Deref for NostrConnect {
    type Target = client::NostrConnect;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<client::NostrConnect> for NostrConnect {
    fn from(inner: client::NostrConnect) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl NostrConnect {
    /// Construct Nostr Connect client
    #[uniffi::constructor]
    pub fn new(
        uri: &NostrConnectURI,
        app_keys: &Keys,
        timeout: Duration,
        opts: Option<Arc<RelayOptions>>,
    ) -> Result<Self> {
        Ok(Self {
            inner: client::NostrConnect::new(
                uri.deref().clone(),
                app_keys.deref().clone(),
                timeout,
                opts.map(|k| k.as_ref().deref().clone()),
            )?,
        })
    }

    /// Get signer relays
    pub fn relays(&self) -> Vec<String> {
        self.inner.relays().iter().map(|u| u.to_string()).collect()
    }

    /// Get `bunker` URI
    pub async fn bunker_uri(&self) -> Result<NostrConnectURI> {
        Ok(self.inner.bunker_uri().await?.into())
    }

    pub async fn get_public_key(&self) -> Result<PublicKey> {
        Ok(self.inner.get_public_key().await?.into())
    }

    pub async fn sign_event(&self, unsigned_event: &UnsignedEvent) -> Result<Event> {
        Ok(self
            .inner
            .sign_event(unsigned_event.deref().clone())
            .await?
            .into())
    }

    pub async fn nip04_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String> {
        Ok(self
            .inner
            .nip04_encrypt(public_key.deref(), content)
            .await?)
    }

    pub async fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        encrypted_content: &str,
    ) -> Result<String> {
        Ok(self
            .inner
            .nip04_decrypt(public_key.deref(), encrypted_content)
            .await?)
    }

    pub async fn nip44_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String> {
        Ok(self
            .inner
            .nip44_encrypt(public_key.deref(), content)
            .await?)
    }

    pub async fn nip44_decrypt(&self, public_key: &PublicKey, payload: &str) -> Result<String> {
        Ok(self
            .inner
            .nip44_decrypt(public_key.deref(), payload)
            .await?)
    }
}
