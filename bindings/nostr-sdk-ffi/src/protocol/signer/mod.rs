// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::borrow::Cow;
use std::ops::Deref;
use std::sync::Arc;

use nostr::signer;
use nostr::signer::IntoNostrSigner;
use uniffi::{Enum, Object};

use crate::connect::NostrConnect;

pub mod custom;

use self::custom::{CustomNostrSigner, IntermediateCustomNostrSigner};
use super::event::{Event, UnsignedEvent};
use super::key::PublicKey;
use crate::error::Result;
use crate::protocol::key::Keys;

#[derive(Enum)]
pub enum SignerBackend {
    /// Secret key
    Keys,
    /// Browser extension (NIP07)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/07.md>
    BrowserExtension,
    /// Nostr Connect (NIP46)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/46.md>
    NostrConnect,
    /// Custom
    Custom { backend: String },
}

impl From<signer::SignerBackend<'_>> for SignerBackend {
    fn from(backend: signer::SignerBackend<'_>) -> Self {
        match backend {
            signer::SignerBackend::Keys => Self::Keys,
            signer::SignerBackend::BrowserExtension => Self::BrowserExtension,
            signer::SignerBackend::NostrConnect => Self::NostrConnect,
            signer::SignerBackend::Custom(backend) => Self::Custom {
                backend: backend.into_owned(),
            },
        }
    }
}

impl From<SignerBackend> for signer::SignerBackend<'_> {
    fn from(backend: SignerBackend) -> Self {
        match backend {
            SignerBackend::Keys => Self::Keys,
            SignerBackend::BrowserExtension => Self::BrowserExtension,
            SignerBackend::NostrConnect => Self::NostrConnect,
            SignerBackend::Custom { backend } => Self::Custom(Cow::Owned(backend)),
        }
    }
}

#[derive(Object)]
pub struct NostrSigner {
    inner: Arc<dyn signer::NostrSigner>,
}

impl Deref for NostrSigner {
    type Target = Arc<dyn signer::NostrSigner>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Arc<dyn signer::NostrSigner>> for NostrSigner {
    fn from(inner: Arc<dyn signer::NostrSigner>) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl NostrSigner {
    #[uniffi::constructor]
    pub fn keys(keys: &Keys) -> Self {
        let signer = keys.deref().clone();
        Self {
            inner: signer.into_nostr_signer(),
        }
    }

    #[uniffi::constructor]
    pub fn nostr_connect(connect: &NostrConnect) -> Self {
        let signer = connect.deref().clone();
        Self {
            inner: signer.into_nostr_signer(),
        }
    }

    #[uniffi::constructor]
    pub fn custom(custom: Arc<dyn CustomNostrSigner>) -> Self {
        let signer = IntermediateCustomNostrSigner { inner: custom };
        Self {
            inner: signer.into_nostr_signer(),
        }
    }

    pub fn backend(&self) -> SignerBackend {
        self.inner.backend().into()
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
