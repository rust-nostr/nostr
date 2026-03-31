// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Signer

use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use core::any::Any;
use core::fmt::{self, Debug, Display};

use crate::nips::nip04::{AsyncNip04, Nip04};
use crate::nips::nip44::{AsyncNip44, Nip44};
use crate::util::BoxedFuture;
use crate::{Event, PublicKey, UnsignedEvent};

/// Nostr Signer error
#[derive(Debug, PartialEq, Eq)]
pub struct SignerError(String);

impl core::error::Error for SignerError {}

impl fmt::Display for SignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl SignerError {
    /// New signer error
    #[inline]
    pub fn backend<E>(error: E) -> Self
    where
        E: Display,
    {
        Self(error.to_string())
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

/// Nostr signer
pub trait AsyncNostrSigner:
    AsyncGetPublicKey
    + AsyncSignEvent
    + AsyncNip04<Error = SignerError>
    + AsyncNip44<Error = SignerError>
{
    /// Signer backend
    fn backend(&self) -> SignerBackend<'_>;
}

/// Get public key
pub trait AsyncGetPublicKey: Any + Debug + Send + Sync {
    /// Get signer public key
    fn get_public_key(&self) -> BoxedFuture<'_, Result<PublicKey, SignerError>>;
}

/// Sign event
pub trait AsyncSignEvent: Any + Debug + Send + Sync {
    /// Sign an unsigned event
    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<'_, Result<Event, SignerError>>;
}

impl<T> AsyncGetPublicKey for T
where
    T: AsRef<dyn AsyncNostrSigner> + Debug + Send + Sync + 'static,
{
    #[inline]
    fn get_public_key(&self) -> BoxedFuture<'_, Result<PublicKey, SignerError>> {
        self.as_ref().get_public_key()
    }
}

impl<T> AsyncSignEvent for T
where
    T: AsRef<dyn AsyncNostrSigner> + Debug + Send + Sync + 'static,
{
    #[inline]
    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<'_, Result<Event, SignerError>> {
        self.as_ref().sign_event(unsigned)
    }
}

impl<T> AsyncNip04 for T
where
    T: AsRef<dyn AsyncNostrSigner> + Debug + Send + Sync + 'static,
{
    type Error = SignerError;

    #[inline]
    fn nip04_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>> {
        self.as_ref().nip04_encrypt(public_key, content)
    }

    #[inline]
    fn nip04_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        encrypted_content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>> {
        self.as_ref().nip04_decrypt(public_key, encrypted_content)
    }
}

impl<T> AsyncNip44 for T
where
    T: AsRef<dyn AsyncNostrSigner> + Debug + Send + Sync + 'static,
{
    type Error = SignerError;

    #[inline]
    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>> {
        self.as_ref().nip44_encrypt(public_key, content)
    }

    #[inline]
    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        payload: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>> {
        self.as_ref().nip44_decrypt(public_key, payload)
    }
}

/// Get public key
pub trait GetPublicKey: Any + Debug {
    /// Get signer public key
    fn get_public_key(&self) -> Result<PublicKey, SignerError>;
}

/// Sign event
pub trait SignEvent: Any + Debug {
    /// Sign an unsigned event
    fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, SignerError>;
}

/// Nostr signer
pub trait NostrSigner:
    GetPublicKey + SignEvent + Nip04<Error = SignerError> + Nip44<Error = SignerError>
{
    /// Signer backend
    fn backend(&self) -> SignerBackend<'_>;
}

impl<T> GetPublicKey for T
where
    T: AsRef<dyn NostrSigner> + Debug + 'static,
{
    #[inline]
    fn get_public_key(&self) -> Result<PublicKey, SignerError> {
        self.as_ref().get_public_key()
    }
}

impl<T> SignEvent for T
where
    T: AsRef<dyn NostrSigner> + Debug + 'static,
{
    #[inline]
    fn sign_event(&self, unsigned: UnsignedEvent) -> Result<Event, SignerError> {
        self.as_ref().sign_event(unsigned)
    }
}

impl<T> Nip04 for T
where
    T: AsRef<dyn NostrSigner> + Debug + 'static,
{
    type Error = SignerError;

    #[inline]
    fn nip04_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String, SignerError> {
        self.as_ref().nip04_encrypt(public_key, content)
    }

    #[inline]
    fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        encrypted_content: &str,
    ) -> Result<String, SignerError> {
        self.as_ref().nip04_decrypt(public_key, encrypted_content)
    }
}

impl<T> Nip44 for T
where
    T: AsRef<dyn NostrSigner> + Debug + 'static,
{
    type Error = SignerError;

    #[inline]
    fn nip44_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String, Self::Error> {
        self.as_ref().nip44_encrypt(public_key, content)
    }

    #[inline]
    fn nip44_decrypt(&self, public_key: &PublicKey, payload: &str) -> Result<String, Self::Error> {
        self.as_ref().nip44_decrypt(public_key, payload)
    }
}
