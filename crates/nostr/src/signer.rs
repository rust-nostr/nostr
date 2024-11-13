// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Signer

use alloc::borrow::Cow;
use alloc::boxed::Box;
#[cfg(any(not(feature = "std"), feature = "nip04", feature = "nip44"))]
use alloc::string::String;
use alloc::sync::Arc;
use core::fmt;

use async_trait::async_trait;

use crate::{Event, PublicKey, UnsignedEvent};

#[cfg(feature = "std")]
type InnerError = Box<dyn std::error::Error + Send + Sync>;
#[cfg(not(feature = "std"))]
type InnerError = String; // TODO: remove core::error::Error will be stable for MSRV

/// Nostr Signer error
#[derive(Debug)]
pub struct SignerError(InnerError);

impl fmt::Display for SignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SignerError {}

impl SignerError {
    /// New signer error
    #[inline]
    #[cfg(feature = "std")]
    pub fn backend<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(Box::new(error))
    }

    /// New signer error
    #[inline]
    #[cfg(not(feature = "std"))]
    pub fn backend<E>(error: E) -> Self
    where
        E: Into<String>,
    {
        Self(error.into())
    }
}

#[doc(hidden)]
pub trait IntoNostrSigner {
    fn into_nostr_signer(self) -> Arc<dyn NostrSigner>;
}

impl<T> IntoNostrSigner for T
where
    T: NostrSigner + 'static,
{
    fn into_nostr_signer(self) -> Arc<dyn NostrSigner> {
        Arc::new(self)
    }
}

/// Signer backend
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignerBackend<'a> {
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
    Custom(Cow<'a, str>),
}

/// Nostr signer abstraction
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrSigner: AsyncTraitDeps {
    /// Signer backend
    fn backend(&self) -> SignerBackend;

    /// Get signer public key
    async fn get_public_key(&self) -> Result<PublicKey, SignerError>;

    /// Sign an unsigned event
    async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, SignerError>;

    /// NIP04 encrypt (deprecate and unsecure)
    #[cfg(feature = "nip04")]
    async fn nip04_encrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError>;

    /// NIP04 decrypt
    #[cfg(feature = "nip04")]
    async fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        encrypted_content: &str,
    ) -> Result<String, SignerError>;

    /// NIP44 encrypt
    #[cfg(feature = "nip44")]
    async fn nip44_encrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError>;

    /// NIP44 decrypt
    #[cfg(feature = "nip44")]
    async fn nip44_decrypt(
        &self,
        public_key: &PublicKey,
        payload: &str,
    ) -> Result<String, SignerError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrSigner for Arc<dyn NostrSigner> {
    #[inline]
    fn backend(&self) -> SignerBackend {
        self.as_ref().backend()
    }

    #[inline]
    async fn get_public_key(&self) -> Result<PublicKey, SignerError> {
        self.as_ref().get_public_key().await
    }

    #[inline]
    async fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, SignerError> {
        self.as_ref().sign_event(unsigned).await
    }

    #[inline]
    #[cfg(feature = "nip04")]
    async fn nip04_encrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self.as_ref().nip04_encrypt(public_key, content).await
    }

    #[inline]
    #[cfg(feature = "nip04")]
    async fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        encrypted_content: &str,
    ) -> Result<String, SignerError> {
        self.as_ref()
            .nip04_decrypt(public_key, encrypted_content)
            .await
    }

    #[inline]
    #[cfg(feature = "nip44")]
    async fn nip44_encrypt(
        &self,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, SignerError> {
        self.as_ref().nip44_encrypt(public_key, content).await
    }

    #[inline]
    #[cfg(feature = "nip44")]
    async fn nip44_decrypt(
        &self,
        public_key: &PublicKey,
        payload: &str,
    ) -> Result<String, SignerError> {
        self.as_ref().nip44_decrypt(public_key, payload).await
    }
}

/// Alias for `Send` on non-wasm, empty trait (implemented by everything) on
/// wasm.
#[cfg(not(target_arch = "wasm32"))]
pub trait SendOutsideWasm: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> SendOutsideWasm for T {}

/// Alias for `Send` on non-wasm, empty trait (implemented by everything) on
/// wasm.
#[cfg(target_arch = "wasm32")]
pub trait SendOutsideWasm {}
#[cfg(target_arch = "wasm32")]
impl<T> SendOutsideWasm for T {}

/// Alias for `Sync` on non-wasm, empty trait (implemented by everything) on
/// wasm.
#[cfg(not(target_arch = "wasm32"))]
pub trait SyncOutsideWasm: Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync> SyncOutsideWasm for T {}

/// Alias for `Sync` on non-wasm, empty trait (implemented by everything) on
/// wasm.
#[cfg(target_arch = "wasm32")]
pub trait SyncOutsideWasm {}
#[cfg(target_arch = "wasm32")]
impl<T> SyncOutsideWasm for T {}

/// Super trait that is used for our store traits, this trait will differ if
/// it's used on WASM. WASM targets will not require `Send` and `Sync` to have
/// implemented, while other targets will.
pub trait AsyncTraitDeps: fmt::Debug + SendOutsideWasm + SyncOutsideWasm {}
impl<T: fmt::Debug + SendOutsideWasm + SyncOutsideWasm> AsyncTraitDeps for T {}
