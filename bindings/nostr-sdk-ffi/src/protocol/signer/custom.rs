// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::sync::Arc;

use crate::error::Result;
use crate::protocol::signer::SignerBackend;
use crate::protocol::{Event, PublicKey, UnsignedEvent};

// NOTE: for some weird reason the `Arc<T>` as output must be wrapped inside a `Vec<T>` or an `Option<T>`
// otherwise compilation will fail.
#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait CustomNostrSigner: Send + Sync {
    fn backend(&self) -> SignerBackend;

    /// Get signer public key
    async fn get_public_key(&self) -> Result<Option<Arc<PublicKey>>>;

    /// Sign an unsigned event
    async fn sign_event(&self, unsigned_event: Arc<UnsignedEvent>) -> Result<Option<Arc<Event>>>;

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

pub(super) struct IntermediateCustomNostrSigner {
    pub(super) inner: Arc<dyn CustomNostrSigner>,
}

impl fmt::Debug for IntermediateCustomNostrSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntermediateCustomNostrSigner").finish()
    }
}

mod inner {
    use std::ops::Deref;
    use std::sync::Arc;

    use async_trait::async_trait;
    use nostr::prelude::*;

    use super::IntermediateCustomNostrSigner;
    use crate::error::NostrSdkError;

    #[async_trait]
    impl NostrSigner for IntermediateCustomNostrSigner {
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
