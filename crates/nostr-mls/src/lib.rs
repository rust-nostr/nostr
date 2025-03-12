#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

use openmls::prelude::*;
use openmls::storage::StorageProvider;
use openmls_rust_crypto::RustCrypto;

mod constant;
pub mod error;
pub mod extension;
pub mod groups;
pub mod key_package;
pub mod prelude;
pub mod welcomes;

use self::constant::{DEFAULT_CIPHERSUITE, REQUIRED_EXTENSIONS};
pub use self::error::Error;

/// The main struct for the Nostr MLS implementation.
#[derive(Debug)]
pub struct NostrMls<Storage>
where
    Storage: StorageProvider,
{
    /// The ciphersuite to use
    pub ciphersuite: Ciphersuite,
    /// The required extensions
    pub extensions: Vec<ExtensionType>,
    /// An implementation of the OpenMls provider trait
    pub provider: NostrMlsProvider<Storage>,
}

/// The provider struct for Nostr MLS that implements the OpenMLS Provider trait.
#[derive(Debug)]
pub struct NostrMlsProvider<Storage> {
    crypto: RustCrypto,
    storage: Storage,
}

impl<Storage> OpenMlsProvider for NostrMlsProvider<Storage>
where
    Storage: StorageProvider,
{
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = Storage;

    fn storage(&self) -> &Self::StorageProvider {
        &self.storage
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
    Storage: StorageProvider,
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
    pub fn capabilities(&self) -> Capabilities {
        Capabilities::new(
            None,
            Some(&[self.ciphersuite]),
            Some(&self.extensions),
            None,
            None,
        )
    }

    pub(crate) fn ciphersuite_value(&self) -> u16 {
        self.ciphersuite.into()
    }

    pub(crate) fn extensions_value(&self) -> String {
        self.extensions
            .iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<String>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use openmls_memory_storage::MemoryStorage;

    use super::*;

    pub fn create_test_nostr_mls() -> NostrMls<MemoryStorage> {
        NostrMls::new(MemoryStorage::default())
    }
}
