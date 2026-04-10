use alloc::string::String;
use core::any::Any;
use core::fmt::Debug;

use crate::key::PublicKey;
use crate::util::BoxedFuture;

/// Synchronous NIP-04
pub trait Nip04: Any + Debug {
    /// NIP-04 error
    type Error: core::error::Error;

    /// Encrypts synchronously using NIP-04.
    ///
    /// **NIP-04 is considered deprecated and unsecure!**
    fn nip04_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String, Self::Error>;

    /// Decrypts synchronously a NIP-04 payload.
    fn nip04_decrypt(
        &self,
        public_key: &PublicKey,
        encrypted_content: &str,
    ) -> Result<String, Self::Error>;
}

/// Asynchronous NIP-04
pub trait AsyncNip04: Any + Debug + Send + Sync {
    /// NIP-04 error
    type Error: core::error::Error;

    /// Encrypts asynchronously using NIP-04.
    ///
    /// **NIP-04 is considered deprecated and unsecure!**
    fn nip04_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>>;

    /// Decrypts asynchronously a NIP-04 payload.
    fn nip04_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        encrypted_content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>>;
}
