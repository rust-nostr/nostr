// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

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
    pub fn keys(keys: &Keys) -> Self {
        Self {
            inner: signer::NostrSigner::Keys(keys.deref().clone()),
        }
    }

    #[uniffi::constructor]
    pub fn nip46(nip46: &Nip46Signer) -> Self {
        Self {
            inner: signer::NostrSigner::nip46(nip46.deref().clone()),
        }
    }

    /// Get signer public key
    pub fn public_key(&self) -> Result<PublicKey> {
        block_on(async move { Ok(self.inner.public_key().await?.into_owned().into()) })
    }

    pub fn sign_event_builder(&self, builder: &EventBuilder) -> Result<Event> {
        block_on(async move {
            Ok(self
                .inner
                .sign_event_builder(builder.deref().clone())
                .await?
                .into())
        })
    }

    pub fn sign_event(&self, unsigned_event: &UnsignedEvent) -> Result<Event> {
        block_on(async move {
            Ok(self
                .inner
                .sign_event(unsigned_event.deref().clone())
                .await?
                .into())
        })
    }

    pub fn nip04_encrypt(&self, public_key: &PublicKey, content: String) -> Result<String> {
        block_on(async move {
            Ok(self
                .inner
                .nip04_encrypt(public_key.deref(), content)
                .await?)
        })
    }

    pub fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        encrypted_content: String,
    ) -> Result<String> {
        block_on(async move {
            Ok(self
                .inner
                .nip04_decrypt(public_key.deref(), encrypted_content)
                .await?)
        })
    }

    pub fn nip44_encrypt(&self, public_key: &PublicKey, content: String) -> Result<String> {
        block_on(async move {
            Ok(self
                .inner
                .nip44_encrypt(public_key.deref(), content)
                .await?)
        })
    }

    pub fn nip44_decrypt(&self, public_key: &PublicKey, content: String) -> Result<String> {
        block_on(async move {
            Ok(self
                .inner
                .nip44_decrypt(public_key.deref(), content)
                .await?)
        })
    }
}
