// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Database

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use async_trait::async_trait;
use nostr::{Event, EventId, Filter, Url};

mod error;
pub mod memory;

pub use self::error::DatabaseError;

/// Backend
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

/// A type-erased [`StateStore`].
pub type DynNostrDatabase = dyn NostrDatabase<Err = DatabaseError>;

/// Nostr SDK Database
#[async_trait]
pub trait NostrDatabase: AsyncTraitDeps {
    /// Error
    type Err;

    /// Name of the backend database used (ex. rocksdb, lmdb, sqlite, indexeddb, ...)
    fn backend(&self) -> Backend;

    /// Save [`Event`] into store
    async fn save_event(&self, event: &Event) -> Result<(), Self::Err>;

    /// Check if [`EventId`] was already seen
    async fn event_id_already_seen(&self, event_id: EventId) -> Result<bool, Self::Err>;

    /// Save [`EventId`] seen by relay
    ///
    /// Useful for NIP65 (gossip)
    async fn save_event_id_seen_by_relay(
        &self,
        event_id: EventId,
        relay_url: Url,
    ) -> Result<(), Self::Err>;

    /// Get list of relays that have seen the [`EventId`]
    async fn event_recently_seen_on_relays(&self, event_id: EventId)
        -> Result<Vec<Url>, Self::Err>;

    /// Query store with filters
    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err>;

    /// Get event IDs by filters
    ///
    /// Uuseful for negentropy reconciliation
    async fn event_ids_by_filters(&self, filters: Vec<Filter>) -> Result<Vec<EventId>, Self::Err>;
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
pub trait AsyncTraitDeps: std::fmt::Debug + SendOutsideWasm + SyncOutsideWasm {}
impl<T: std::fmt::Debug + SendOutsideWasm + SyncOutsideWasm> AsyncTraitDeps for T {}
