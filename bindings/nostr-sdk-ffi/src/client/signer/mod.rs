// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::{Event, EventBuilder, Keys, PublicKey, UnsignedEvent};
use nostr_sdk::{block_on, signer};
use uniffi::Object;

pub mod nip46;

use self::nip46::Nip46Signer;
use crate::error::Result;

#[derive(Object)]
pub struct NostrSigner {
    inner: signer::NostrSigner,
}

impl Deref for NostrSigner {
    type Target = signer::NostrSigner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<signer::NostrSigner> for NostrSigner {
    fn from(inner: signer::NostrSigner) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl NostrSigner {
    #[uniffi::constructor]
    pub fn keys(keys: Arc<Keys>) -> Self {
        Self {
            inner: signer::NostrSigner::Keys(keys.as_ref().deref().clone()),
        }
    }

    #[uniffi::constructor]
    pub fn nip46(nip46: Arc<Nip46Signer>) -> Self {
        Self {
            inner: signer::NostrSigner::nip46(nip46.as_ref().deref().clone()),
        }
    }

    /// Get signer public key
    pub fn public_key(&self) -> Result<Arc<PublicKey>> {
        block_on(async move { Ok(Arc::new(self.inner.public_key().await?.into())) })
    }

    pub fn sign_event_builder(&self, builder: Arc<EventBuilder>) -> Result<Arc<Event>> {
        block_on(async move {
            Ok(Arc::new(
                self.inner
                    .sign_event_builder(builder.as_ref().deref().clone())
                    .await?
                    .into(),
            ))
        })
    }

    pub fn sign_event(&self, unsigned_event: Arc<UnsignedEvent>) -> Result<Arc<Event>> {
        block_on(async move {
            Ok(Arc::new(
                self.inner
                    .sign_event(unsigned_event.as_ref().deref().clone())
                    .await?
                    .into(),
            ))
        })
    }

    pub fn nip04_encrypt(&self, public_key: Arc<PublicKey>, content: String) -> Result<String> {
        block_on(async move { Ok(self.inner.nip04_encrypt(**public_key, content).await?) })
    }

    pub fn nip04_decrypt(
        &self,
        public_key: Arc<PublicKey>,
        encrypted_content: String,
    ) -> Result<String> {
        block_on(async move {
            Ok(self
                .inner
                .nip04_decrypt(**public_key, encrypted_content)
                .await?)
        })
    }

    pub fn nip44_encrypt(&self, public_key: Arc<PublicKey>, content: String) -> Result<String> {
        block_on(async move { Ok(self.inner.nip44_encrypt(**public_key, content).await?) })
    }

    pub fn nip44_decrypt(&self, public_key: Arc<PublicKey>, content: String) -> Result<String> {
        block_on(async move { Ok(self.inner.nip44_decrypt(**public_key, content).await?) })
    }
}
