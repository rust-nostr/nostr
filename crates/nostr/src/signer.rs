// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Signer

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::sync::Arc;
use core::fmt;

use crate::util::BoxedFuture;
use crate::{Event, PublicKey, UnsignedEvent};

/// Nostr Signer error
#[derive(Debug, PartialEq, Eq)]
pub struct SignerError(String);

#[cfg(feature = "std")]
impl std::error::Error for SignerError {}

impl fmt::Display for SignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SignerError {
    /// New signer error
    #[inline]
    #[cfg(feature = "std")]
    pub fn backend<E>(error: E) -> Self
    where
        E: std::error::Error,
    {
        Self(error.to_string())
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

impl<S> From<S> for SignerError
where
    S: Into<String>,
{
    fn from(error: S) -> Self {
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
pub trait NostrSigner: fmt::Debug + Send + Sync {
    /// Signer backend
    fn backend(&self) -> SignerBackend;

    /// Get signer public key
    fn get_public_key(&self) -> BoxedFuture<Result<PublicKey, SignerError>>;

    /// Sign an unsigned event
    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<Result<Event, SignerError>>;

    /// NIP04 encrypt (deprecate and unsecure)
    fn nip04_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>>;

    /// NIP04 decrypt
    fn nip04_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        encrypted_content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>>;

    /// NIP44 encrypt
    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>>;

    /// NIP44 decrypt
    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        payload: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>>;
}

impl NostrSigner for Arc<dyn NostrSigner> {
    #[inline]
    fn backend(&self) -> SignerBackend {
        self.as_ref().backend()
    }

    #[inline]
    fn get_public_key(&self) -> BoxedFuture<Result<PublicKey, SignerError>> {
        self.as_ref().get_public_key()
    }

    #[inline]
    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<Result<Event, SignerError>> {
        self.as_ref().sign_event(unsigned)
    }

    #[inline]
    fn nip04_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        self.as_ref().nip04_encrypt(public_key, content)
    }

    #[inline]
    fn nip04_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        encrypted_content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        self.as_ref().nip04_decrypt(public_key, encrypted_content)
    }

    #[inline]
    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        self.as_ref().nip44_encrypt(public_key, content)
    }

    #[inline]
    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        payload: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        self.as_ref().nip44_decrypt(public_key, payload)
    }
}
