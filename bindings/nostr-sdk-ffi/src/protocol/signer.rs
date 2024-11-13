// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use nostr::signer;
use uniffi::Enum;

use super::event::{Event, UnsignedEvent};
use super::key::PublicKey;
use crate::error::Result;

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

impl<'a> From<signer::SignerBackend<'a>> for SignerBackend {
    fn from(backend: signer::SignerBackend<'a>) -> Self {
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

impl<'a> From<SignerBackend> for signer::SignerBackend<'a> {
    fn from(backend: SignerBackend) -> Self {
        match backend {
            SignerBackend::Keys => Self::Keys,
            SignerBackend::BrowserExtension => Self::BrowserExtension,
            SignerBackend::NostrConnect => Self::NostrConnect,
            SignerBackend::Custom { backend } => Self::Custom(Cow::Owned(backend)),
        }
    }
}

// NOTE: for some weird reason the `Arc<T>` as output must be wrapped inside a `Vec<T>` or an `Option<T>`
// otherwise compilation will fail.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait NostrSigner: Send + Sync {
    fn backend(&self) -> SignerBackend;

    /// Get signer public key
    async fn get_public_key(&self) -> Result<Option<Arc<PublicKey>>>;

    /// Sign an unsigned event
    async fn sign_event(&self, unsigned: Arc<UnsignedEvent>) -> Result<Option<Arc<Event>>>;

    /// NIP04 encrypt (deprecate and unsecure)
    async fn nip04_encrypt(&self, public_key: Arc<PublicKey>, content: String) -> Result<String>;

    /// NIP04 decrypt
    async fn nip04_decrypt(
        &self,
        public_key: Arc<PublicKey>,
        encrypted_content: String,
    ) -> Result<String>;

    /// NIP44 encrypt
    async fn nip44_encrypt(&self, public_key: Arc<PublicKey>, content: String) -> Result<String>;

    /// NIP44 decrypt
    async fn nip44_decrypt(&self, public_key: Arc<PublicKey>, payload: String) -> Result<String>;
}

pub struct NostrSignerFFI2Rust {
    pub(super) inner: Arc<dyn NostrSigner>,
}

impl fmt::Debug for NostrSignerFFI2Rust {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NostrSignerFFI2Rust").finish()
    }
}

impl NostrSignerFFI2Rust {
    pub fn new(inner: Arc<dyn NostrSigner>) -> Self {
        Self { inner }
    }
}

pub struct NostrSignerRust2FFI {
    pub(super) inner: Arc<dyn nostr::signer::NostrSigner>,
}

impl NostrSignerRust2FFI {
    pub fn new(inner: Arc<dyn nostr::signer::NostrSigner>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl NostrSigner for NostrSignerRust2FFI {
    fn backend(&self) -> SignerBackend {
        self.inner.backend().into()
    }

    async fn get_public_key(&self) -> Result<Option<Arc<PublicKey>>> {
        Ok(Some(Arc::new(self.inner.get_public_key().await?.into())))
    }

    async fn sign_event(&self, unsigned: Arc<UnsignedEvent>) -> Result<Option<Arc<Event>>> {
        Ok(Some(Arc::new(
            self.inner
                .sign_event(unsigned.as_ref().deref().clone())
                .await?
                .into(),
        )))
    }

    async fn nip04_encrypt(&self, public_key: Arc<PublicKey>, content: String) -> Result<String> {
        Ok(self
            .inner
            .nip04_encrypt(public_key.as_ref().deref(), &content)
            .await?)
    }

    async fn nip04_decrypt(
        &self,
        public_key: Arc<PublicKey>,
        encrypted_content: String,
    ) -> Result<String> {
        Ok(self
            .inner
            .nip04_decrypt(public_key.as_ref().deref(), &encrypted_content)
            .await?)
    }

    async fn nip44_encrypt(&self, public_key: Arc<PublicKey>, content: String) -> Result<String> {
        Ok(self
            .inner
            .nip44_encrypt(public_key.as_ref().deref(), &content)
            .await?)
    }

    async fn nip44_decrypt(&self, public_key: Arc<PublicKey>, payload: String) -> Result<String> {
        Ok(self
            .inner
            .nip44_decrypt(public_key.as_ref().deref(), &payload)
            .await?)
    }
}

mod inner {
    use std::ops::Deref;
    use std::sync::Arc;

    use async_trait::async_trait;
    use nostr::prelude::*;

    use super::NostrSignerFFI2Rust;
    use crate::error::NostrSdkError;

    #[async_trait]
    impl NostrSigner for NostrSignerFFI2Rust {
        fn backend(&self) -> SignerBackend {
            self.inner.backend().into()
        }

        async fn get_public_key(&self) -> Result<PublicKey, SignerError> {
            let public_key = self
                .inner
                .get_public_key()
                .await
                .map_err(SignerError::backend)?
                .ok_or_else(|| {
                    SignerError::backend(NostrSdkError::Generic(String::from(
                        "Received None instead of public key",
                    )))
                })?;
            Ok(**public_key)
        }

        async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, SignerError> {
            let unsigned = Arc::new(unsigned.into());
            let event = self
                .inner
                .sign_event(unsigned)
                .await
                .map_err(SignerError::backend)?
                .ok_or_else(|| {
                    SignerError::backend(NostrSdkError::Generic(String::from(
                        "Received None instead of event",
                    )))
                })?;
            Ok(event.as_ref().deref().clone())
        }

        async fn nip04_encrypt(
            &self,
            public_key: &PublicKey,
            content: &str,
        ) -> Result<String, SignerError> {
            let public_key = Arc::new((*public_key).into());
            self.inner
                .nip04_encrypt(public_key, content.to_string())
                .await
                .map_err(SignerError::backend)
        }

        async fn nip04_decrypt(
            &self,
            public_key: &PublicKey,
            encrypted_content: &str,
        ) -> Result<String, SignerError> {
            let public_key = Arc::new((*public_key).into());
            self.inner
                .nip04_decrypt(public_key, encrypted_content.to_string())
                .await
                .map_err(SignerError::backend)
        }

        async fn nip44_encrypt(
            &self,
            public_key: &PublicKey,
            content: &str,
        ) -> Result<String, SignerError> {
            let public_key = Arc::new((*public_key).into());
            self.inner
                .nip44_encrypt(public_key, content.to_string())
                .await
                .map_err(SignerError::backend)
        }

        async fn nip44_decrypt(
            &self,
            public_key: &PublicKey,
            payload: &str,
        ) -> Result<String, SignerError> {
            let public_key = Arc::new((*public_key).into());
            self.inner
                .nip44_decrypt(public_key, payload.to_string())
                .await
                .map_err(SignerError::backend)
        }
    }
}
