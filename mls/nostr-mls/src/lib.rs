//! A Rust implementation of the Nostr Message Layer Security (MLS) protocol
//!
//! This crate provides functionality for implementing secure group messaging in Nostr using the MLS protocol.
//! It handles group creation, member management, message encryption/decryption, key management, and storage of groups and messages.
//! The implementation follows the MLS specification while integrating with Nostr's event system.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

use nostr_mls_storage::NostrMlsStorageProvider;
use openmls::prelude::*;
use openmls_rust_crypto::RustCrypto;

mod constant;
pub mod error;
pub mod extension;
pub mod groups;
pub mod key_packages;
pub mod messages;
pub mod prelude;
mod util;
pub mod welcomes;

use self::constant::{DEFAULT_CIPHERSUITE, REQUIRED_EXTENSIONS};
pub use self::error::Error;

/// The main struct for the Nostr MLS implementation.
///
/// This struct provides the core functionality for MLS operations in Nostr:
/// - Group management (creation, updates, member management)
/// - Message handling (encryption, decryption, processing)
/// - Key management (key packages, welcome messages)
///
/// It uses a generic storage provider that implements the `NostrMlsStorageProvider` trait,
/// allowing for flexible storage backends.
#[derive(Debug)]
pub struct NostrMls<Storage>
where
    Storage: NostrMlsStorageProvider,
{
    /// The MLS ciphersuite used for cryptographic operations
    pub ciphersuite: Ciphersuite,
    /// Required MLS extensions for Nostr functionality
    pub extensions: Vec<ExtensionType>,
    /// The OpenMLS provider implementation for cryptographic and storage operations
    pub provider: NostrMlsProvider<Storage>,
}

/// Provider implementation for OpenMLS that integrates with Nostr.
///
/// This struct implements the OpenMLS Provider trait, providing:
/// - Cryptographic operations through RustCrypto
/// - Storage operations through the generic Storage type
/// - Random number generation through RustCrypto
#[derive(Debug)]
pub struct NostrMlsProvider<Storage>
where
    Storage: NostrMlsStorageProvider,
{
    crypto: RustCrypto,
    storage: Storage,
}

impl<Storage> OpenMlsProvider for NostrMlsProvider<Storage>
where
    Storage: NostrMlsStorageProvider,
{
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = Storage::OpenMlsStorageProvider;

    fn storage(&self) -> &Self::StorageProvider {
        self.storage.openmls_storage()
    }

    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }

    fn rand(&self) -> &Self::RandProvider {
        &self.crypto
    }
}

impl<Storage> NostrMls<Storage>
where
    Storage: NostrMlsStorageProvider,
{
    /// Construct new nostr MLS instance
    pub fn new(storage: Storage) -> Self {
        Self {
            ciphersuite: DEFAULT_CIPHERSUITE,
            extensions: REQUIRED_EXTENSIONS.to_vec(),
            provider: NostrMlsProvider {
                crypto: RustCrypto::default(),
                storage,
            },
        }
    }

    /// Get nostr MLS capabilities
    #[inline]
    pub(crate) fn capabilities(&self) -> Capabilities {
        Capabilities::new(
            None,
            Some(&[self.ciphersuite]),
            Some(&self.extensions),
            None,
            None,
        )
    }

    /// Get nostr mls group's required capabilities extension
    #[inline]
    pub(crate) fn required_capabilitie_extension(&self) -> Extension {
        Extension::RequiredCapabilities(RequiredCapabilitiesExtension::new(
            &self.extensions,
            &[],
            &[],
        ))
    }

    /// Get the ciphersuite value
    pub(crate) fn ciphersuite_value(&self) -> u16 {
        self.ciphersuite.into()
    }

    /// Get the extensions value
    pub(crate) fn extensions_value(&self) -> String {
        self.extensions
            .iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<String>>()
            .join(",")
    }

    /// Get the storage provider
    pub(crate) fn storage(&self) -> &Storage {
        &self.provider.storage
    }
}

/// Tests module for nostr-mls
#[cfg(test)]
pub mod tests {
    use nostr_mls_memory_storage::NostrMlsMemoryStorage;

    use super::*;

    /// Create a test NostrMls instance with an in-memory storage provider
    pub fn create_test_nostr_mls() -> NostrMls<NostrMlsMemoryStorage> {
        NostrMls::new(NostrMlsMemoryStorage::default())
    }
}
