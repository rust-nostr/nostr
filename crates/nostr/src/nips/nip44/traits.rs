// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use alloc::string::String;
use core::any::Any;
use core::fmt::Debug;

use crate::key::PublicKey;
use crate::util::BoxedFuture;

/// Synchronous NIP-44
pub trait Nip44: Any + Debug {
    /// NIP-44 error
    type Error: core::error::Error;

    /// Encrypts synchronously using NIP-44.
    ///
    /// The NIP-44 version is chosen by the implementation.
    fn nip44_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String, Self::Error>;

    /// Decrypts synchronously a NIP-44 payload.
    fn nip44_decrypt(&self, public_key: &PublicKey, payload: &str) -> Result<String, Self::Error>;
}

/// Asynchronous NIP-44
pub trait AsyncNip44: Any + Debug + Send + Sync {
    /// NIP-44 error
    type Error: core::error::Error;

    /// Encrypts asynchronously using NIP-44.
    ///
    /// The NIP-44 version is chosen by the implementation.
    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>>;

    /// Decrypts asynchronously a NIP-44 payload.
    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        payload: &'a str,
    ) -> BoxedFuture<'a, Result<String, Self::Error>>;
}
