// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

use std::sync::Arc;

pub use async_trait::async_trait;
pub use nostr;

mod collections;
mod error;
mod events;
#[cfg(feature = "flatbuf")]
pub mod flatbuffers;
pub mod memory;
pub mod prelude;
pub mod profile;

pub use self::collections::events::{Events, QueryEvent};
pub use self::error::DatabaseError;
pub use self::events::helper::{DatabaseEventResult, DatabaseHelper};
pub use self::events::{
    DatabaseEventStatus, IntoNostrEventsDatabase, NostrEventsDatabase, NostrEventsDatabaseExt,
    RejectedReason, SaveEventStatus,
};
#[cfg(feature = "flatbuf")]
pub use self::flatbuffers::{FlatBufferBuilder, FlatBufferDecode, FlatBufferEncode};
pub use self::memory::{MemoryDatabase, MemoryDatabaseOptions};
pub use self::profile::Profile;

/// Backend
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    /// Memory
    Memory,
    /// RocksDB
    RocksDB,
    /// Lightning Memory-Mapped Database
    LMDB,
    /// SQLite
    SQLite,
    /// IndexedDB
    IndexedDB,
    /// Custom
    Custom(String),
}

impl Backend {
    /// Check if it's a persistent backend
    ///
    /// All values different from [`Backend::Memory`] are considered persistent
    pub fn is_persistent(&self) -> bool {
        !matches!(self, Self::Memory)
    }
}

#[doc(hidden)]
pub trait IntoNostrDatabase {
    fn into_nostr_database(self) -> Arc<dyn NostrDatabase>;
}

impl IntoNostrDatabase for Arc<dyn NostrDatabase> {
    fn into_nostr_database(self) -> Arc<dyn NostrDatabase> {
        self
    }
}

impl<T> IntoNostrDatabase for T
where
    T: NostrDatabase + Sized + 'static,
{
    fn into_nostr_database(self) -> Arc<dyn NostrDatabase> {
        Arc::new(self)
    }
}

impl<T> IntoNostrDatabase for Arc<T>
where
    T: NostrDatabase + 'static,
{
    fn into_nostr_database(self) -> Arc<dyn NostrDatabase> {
        self
    }
}

/// Nostr Database
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrDatabase: NostrEventsDatabase {
    /// Name of the backend database used
    fn backend(&self) -> Backend;

    /// Wipe all data
    async fn wipe(&self) -> Result<(), DatabaseError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_is_persistent() {
        assert!(!Backend::Memory.is_persistent());
        assert!(Backend::RocksDB.is_persistent());
        assert!(Backend::LMDB.is_persistent());
        assert!(Backend::SQLite.is_persistent());
        assert!(Backend::IndexedDB.is_persistent());
        assert!(Backend::Custom("custom".to_string()).is_persistent());
    }
}
