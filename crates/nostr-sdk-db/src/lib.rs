// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Database

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use std::collections::HashSet;

use async_trait::async_trait;
use nostr::{Event, EventId, Filter, Timestamp, Url};

mod error;
pub mod memory;
mod options;

pub use self::error::DatabaseError;
pub use self::options::DatabaseOptions;

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

    /// Database options
    fn opts(&self) -> DatabaseOptions;

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err>;

    /// Check if [`EventId`] has already been seen
    async fn has_event_already_been_seen(&self, event_id: EventId) -> Result<bool, Self::Err>;

    /// Set [`EventId`] as seen
    ///
    /// Optionally, save also the relay url where the event has been seen (useful for NIP65, aka gossip)
    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: Option<Url>,
    ) -> Result<(), Self::Err>;

    /// Set multiple [`EventId`] as seen
    ///
    /// Optionally, save also the relay url where the event has been seen (useful for NIP65, aka gossip)
    async fn event_ids_seen(
        &self,
        event_ids: Vec<EventId>,
        relay_url: Option<Url>,
    ) -> Result<(), Self::Err>;

    /// Get list of relays that have seen the [`EventId`]
    async fn event_recently_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err>;

    /// Get [`Event`] by [`EventId`]
    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err>;

    /// Query store with filters
    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err>;

    /// Get event IDs by filters
    async fn event_ids_by_filters(&self, filters: Vec<Filter>) -> Result<Vec<EventId>, Self::Err>;

    /// Get `negentropy` items
    async fn negentropy_items(
        &self,
        filter: &Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err>;

    /// Wipe all data
    async fn wipe(&self) -> Result<(), Self::Err>;
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
