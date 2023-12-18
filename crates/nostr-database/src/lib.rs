// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use core::fmt;
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

pub use async_trait::async_trait;
pub use nostr;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Event, EventId, Filter, JsonUtil, Kind, Metadata, Timestamp, Url};
use rayon::prelude::*;

mod error;
#[cfg(feature = "flatbuf")]
pub mod flatbuffers;
pub mod index;
pub mod memory;
mod options;
pub mod profile;
mod raw;

pub use self::error::DatabaseError;
#[cfg(feature = "flatbuf")]
pub use self::flatbuffers::{FlatBufferBuilder, FlatBufferDecode, FlatBufferEncode};
pub use self::index::{DatabaseIndexes, EventIndexResult};
pub use self::memory::MemoryDatabase;
pub use self::options::DatabaseOptions;
pub use self::profile::Profile;
pub use self::raw::RawEvent;

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

/// A type-erased [`NostrDatabase`].
pub type DynNostrDatabase = dyn NostrDatabase<Err = DatabaseError>;

/// A type that can be type-erased into `Arc<dyn NostrDatabase>`.
pub trait IntoNostrDatabase {
    #[doc(hidden)]
    fn into_nostr_database(self) -> Arc<DynNostrDatabase>;
}

impl IntoNostrDatabase for Arc<DynNostrDatabase> {
    fn into_nostr_database(self) -> Arc<DynNostrDatabase> {
        self
    }
}

impl<T> IntoNostrDatabase for T
where
    T: NostrDatabase + Sized + 'static,
{
    fn into_nostr_database(self) -> Arc<DynNostrDatabase> {
        Arc::new(EraseNostrDatabaseError(self))
    }
}

// Turns a given `Arc<T>` into `Arc<DynNostrDatabase>` by attaching the
// NostrDatabase impl vtable of `EraseNostrDatabaseError<T>`.
impl<T> IntoNostrDatabase for Arc<T>
where
    T: NostrDatabase + 'static,
{
    fn into_nostr_database(self) -> Arc<DynNostrDatabase> {
        let ptr: *const T = Arc::into_raw(self);
        let ptr_erased = ptr as *const EraseNostrDatabaseError<T>;
        // SAFETY: EraseNostrDatabaseError is repr(transparent) so T and
        //         EraseNostrDatabaseError<T> have the same layout and ABI
        unsafe { Arc::from_raw(ptr_erased) }
    }
}

/// Nostr Database
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrDatabase: AsyncTraitDeps {
    /// Error
    type Err: From<DatabaseError> + Into<DatabaseError>;

    /// Name of the backend database used (ex. rocksdb, lmdb, sqlite, indexeddb, ...)
    fn backend(&self) -> Backend;

    /// Database options
    fn opts(&self) -> DatabaseOptions;

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    ///
    /// **This method assume that [`Event`] was already verified**
    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err>;

    /// Check if [`Event`] has already been saved
    async fn has_event_already_been_saved(&self, event_id: EventId) -> Result<bool, Self::Err>;

    /// Check if [`EventId`] has already been seen
    async fn has_event_already_been_seen(&self, event_id: EventId) -> Result<bool, Self::Err>;

    /// Check if [`EventId`] has been deleted
    async fn has_been_deleted(&self, event_id: EventId) -> Result<bool, Self::Err>;

    /// Set [`EventId`] as seen by relay
    ///
    /// Useful for NIP65 (aka gossip)
    async fn event_id_seen(&self, event_id: EventId, relay_url: Url) -> Result<(), Self::Err>;

    /// Get list of relays that have seen the [`EventId`]
    async fn event_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err>;

    /// Get [`Event`] by [`EventId`]
    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err>;

    /// Count number of [`Event`] found by filters
    ///
    /// Use `Filter::new()` or `Filter::default()` to count all events.
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, Self::Err>;

    /// Query store with filters
    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err>;

    /// Get event IDs by filters
    async fn event_ids_by_filters(&self, filters: Vec<Filter>) -> Result<Vec<EventId>, Self::Err>;

    /// Get `negentropy` items
    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err>;

    /// Wipe all data
    async fn wipe(&self) -> Result<(), Self::Err>;
}

/// Nostr Database Extension
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrDatabaseExt: NostrDatabase {
    /// Get profile metadata
    async fn profile(&self, public_key: XOnlyPublicKey) -> Result<Profile, Self::Err> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        let events: Vec<Event> = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => match Metadata::from_json(&event.content) {
                Ok(metadata) => Ok(Profile::new(public_key, metadata)),
                Err(e) => {
                    tracing::error!("Impossible to deserialize profile metadata: {e}");
                    Ok(Profile::from(public_key))
                }
            },
            None => Ok(Profile::from(public_key)),
        }
    }

    /// Get contact list public keys
    async fn contacts_public_keys(
        &self,
        public_key: XOnlyPublicKey,
    ) -> Result<Vec<XOnlyPublicKey>, Self::Err> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        let events: Vec<Event> = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => Ok(event.public_keys().copied().collect()),
            None => Ok(Vec::new()),
        }
    }

    /// Get contact list with metadata of [`XOnlyPublicKey`]
    #[tracing::instrument(skip_all, level = "trace")]
    async fn contacts(&self, public_key: XOnlyPublicKey) -> Result<BTreeSet<Profile>, Self::Err> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        let events: Vec<Event> = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => {
                // Get contacts metadata
                let filter = Filter::new()
                    .authors(event.public_keys().copied())
                    .kind(Kind::Metadata);
                let mut contacts: BTreeSet<Profile> = self
                    .query(vec![filter])
                    .await?
                    .into_iter()
                    .map(|e| {
                        let metadata: Metadata =
                            Metadata::from_json(&e.content).unwrap_or_default();
                        Profile::new(e.pubkey, metadata)
                    })
                    .collect();

                // Extend with missing public keys
                contacts.par_extend(event.public_keys().par_bridge().copied().map(Profile::from));

                Ok(contacts)
            }
            None => Ok(BTreeSet::new()),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T: NostrDatabase + ?Sized> NostrDatabaseExt for T {}

#[repr(transparent)]
struct EraseNostrDatabaseError<T>(T);

impl<T: fmt::Debug> fmt::Debug for EraseNostrDatabaseError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T: NostrDatabase> NostrDatabase for EraseNostrDatabaseError<T> {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        self.0.backend()
    }

    fn opts(&self) -> DatabaseOptions {
        self.0.opts()
    }

    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        self.0.save_event(event).await.map_err(Into::into)
    }

    async fn has_event_already_been_saved(&self, event_id: EventId) -> Result<bool, Self::Err> {
        self.0
            .has_event_already_been_saved(event_id)
            .await
            .map_err(Into::into)
    }

    async fn has_event_already_been_seen(&self, event_id: EventId) -> Result<bool, Self::Err> {
        self.0
            .has_event_already_been_seen(event_id)
            .await
            .map_err(Into::into)
    }

    async fn has_been_deleted(&self, event_id: EventId) -> Result<bool, Self::Err> {
        self.0.has_been_deleted(event_id).await.map_err(Into::into)
    }

    async fn event_id_seen(&self, event_id: EventId, relay_url: Url) -> Result<(), Self::Err> {
        self.0
            .event_id_seen(event_id, relay_url)
            .await
            .map_err(Into::into)
    }

    async fn event_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err> {
        self.0
            .event_seen_on_relays(event_id)
            .await
            .map_err(Into::into)
    }

    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err> {
        self.0.event_by_id(event_id).await.map_err(Into::into)
    }

    async fn count(&self, filters: Vec<Filter>) -> Result<usize, Self::Err> {
        self.0.count(filters).await.map_err(Into::into)
    }

    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err> {
        self.0.query(filters).await.map_err(Into::into)
    }

    async fn event_ids_by_filters(&self, filters: Vec<Filter>) -> Result<Vec<EventId>, Self::Err> {
        self.0
            .event_ids_by_filters(filters)
            .await
            .map_err(Into::into)
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
        self.0.negentropy_items(filter).await.map_err(Into::into)
    }

    async fn wipe(&self) -> Result<(), Self::Err> {
        self.0.wipe().await.map_err(Into::into)
    }
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
