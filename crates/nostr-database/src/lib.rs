// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr Database

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use core::fmt;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub use async_trait::async_trait;
pub use nostr;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Event, EventId, Filter, JsonUtil, Kind, Metadata, Timestamp, Url};

mod error;
#[cfg(feature = "flatbuf")]
pub mod flatbuffers;
pub mod index;
pub mod memory;
mod options;

pub use self::error::DatabaseError;
#[cfg(feature = "flatbuf")]
pub use self::flatbuffers::{FlatBufferBuilder, FlatBufferDecode, FlatBufferEncode};
pub use self::index::{DatabaseIndexes, EventIndexResult};
pub use self::memory::MemoryDatabase;
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

/// A type-erased [`NostrDatabase`].
pub type DynNostrDatabase = dyn NostrDatabase<Err = DatabaseError>;

/// A type that can be type-erased into `Arc<dyn NostrDatabase>`.
///
/// This trait is not meant to be implemented directly outside
/// `matrix-sdk-crypto`, but it is automatically implemented for everything that
/// implements `NostrDatabase`.
pub trait IntoNostrDatabase {
    #[doc(hidden)]
    fn into_nostr_database(self) -> Arc<DynNostrDatabase>;
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

    /// Count number of [`Event`] stored
    async fn count(&self) -> Result<usize, Self::Err>;

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err>;

    /// Check if [`Event`] has already been saved
    async fn has_event_already_been_saved(&self, event_id: EventId) -> Result<bool, Self::Err>;

    /// Check if [`EventId`] has already been seen
    async fn has_event_already_been_seen(&self, event_id: EventId) -> Result<bool, Self::Err>;

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

    /// Query store with filters
    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err>;

    /// Get event IDs by filters
    async fn event_ids_by_filters(
        &self,
        filters: Vec<Filter>,
    ) -> Result<HashSet<EventId>, Self::Err>;

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
    async fn profile(&self, public_key: XOnlyPublicKey) -> Result<Metadata, Self::Err> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        let events: Vec<Event> = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => Ok(Metadata::from_json(&event.content).map_err(DatabaseError::nostr)?),
            None => Ok(Metadata::default()), // TODO: return an Option?
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
    async fn contacts(
        &self,
        public_key: XOnlyPublicKey,
    ) -> Result<HashMap<XOnlyPublicKey, Metadata>, Self::Err> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        let events: Vec<Event> = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => {
                let public_keys: Vec<XOnlyPublicKey> = event.public_keys().copied().collect();
                let size: usize = public_keys.len();

                let filter = Filter::new()
                    .authors(public_keys.clone())
                    .kind(Kind::Metadata)
                    .limit(size);
                let mut contacts: HashMap<XOnlyPublicKey, Metadata> = self
                    .query(vec![filter])
                    .await?
                    .into_iter()
                    .map(|e| {
                        let metadata: Metadata =
                            Metadata::from_json(&e.content).unwrap_or_default();
                        (e.pubkey, metadata)
                    })
                    .collect();

                for public_key in public_keys.into_iter() {
                    if let Entry::Vacant(e) = contacts.entry(public_key) {
                        e.insert(Metadata::default());
                    }
                }

                Ok(contacts)
            }
            None => Ok(HashMap::new()),
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

    async fn count(&self) -> Result<usize, Self::Err> {
        self.0.count().await.map_err(Into::into)
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

    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err> {
        self.0.query(filters).await.map_err(Into::into)
    }

    async fn event_ids_by_filters(
        &self,
        filters: Vec<Filter>,
    ) -> Result<HashSet<EventId>, Self::Err> {
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
