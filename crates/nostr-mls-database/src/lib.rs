use openmls::storage::StorageProvider;

mod errors;
/// Backend
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    /// Memory
    Memory,
    /// SQLite
    SQLite,
}

impl Backend {
    /// Check if it's a persistent backend
    ///
    /// All values different from [`Backend::Memory`] are considered persistent
    pub fn is_persistent(&self) -> bool {
        !matches!(self, Self::Memory)
    }
}

pub trait NostrMlsStorage: StorageProvider {
    // TODO: Methods that we want to have in the nostr mls storage trait
}

impl<T: StorageProvider> NostrMlsStorage for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_is_persistent() {
        assert!(!Backend::Memory.is_persistent());
        assert!(Backend::SQLite.is_persistent());
    }
}
