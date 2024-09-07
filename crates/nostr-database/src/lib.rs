// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

use core::fmt;
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

pub use async_trait::async_trait;
pub use nostr;
use nostr::{Event, EventId, Filter, JsonUtil, Kind, Metadata, PublicKey, Timestamp, Url};

mod error;
#[cfg(feature = "flatbuf")]
pub mod flatbuffers;
pub mod helper;
pub mod memory;
pub mod prelude;
pub mod profile;
mod tree;
mod util;

pub use self::error::DatabaseError;
#[cfg(feature = "flatbuf")]
pub use self::flatbuffers::{FlatBufferBuilder, FlatBufferDecode, FlatBufferEncode};
pub use self::helper::{DatabaseEventResult, DatabaseHelper};
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

/// Query result order
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Order {
    /// Ascending
    Asc,
    /// Descending (default)
    #[default]
    Desc,
}

/// A type-erased [`NostrDatabase`].
pub type DynNostrDatabase = dyn NostrDatabase;

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
        Arc::new(self)
    }
}

impl<T> IntoNostrDatabase for Arc<T>
where
    T: NostrDatabase + 'static,
{
    fn into_nostr_database(self) -> Arc<DynNostrDatabase> {
        self
    }
}

/// Database event status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DatabaseEventStatus {
    /// The event is saved
    Saved,
    /// The event was deleted
    Deleted,
    /// The event not exists
    NotExistent,
}

/// Nostr Database
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrDatabase: fmt::Debug + Send + Sync {
    /// Name of the backend database used (ex. rocksdb, lmdb, sqlite, indexeddb, ...)
    fn backend(&self) -> Backend;

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    // TODO: return enum saying that event is saved or deleted or replaced and so on or error?
    async fn save_event(&self, event: &Event) -> Result<bool, DatabaseError>;

    /// Check event status
    ///
    /// Check if the event is saved, deleted or not existent.
    async fn check_event(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError>;

    /// Set [`EventId`] as seen by relay
    ///
    /// Useful for NIP65 (aka gossip)
    async fn event_id_seen(&self, event_id: EventId, relay_url: Url) -> Result<(), DatabaseError>;

    /// Get list of relays that have seen the [`EventId`]
    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<Url>>, DatabaseError>;

    /// Get [`Event`] by [`EventId`]
    async fn event_by_id(&self, event_id: &EventId) -> Result<Event, DatabaseError>;

    /// Count number of [`Event`] found by filters
    ///
    /// Use `Filter::new()` or `Filter::default()` to count all events.
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError>;

    /// Query store with filters
    async fn query(&self, filters: Vec<Filter>, order: Order) -> Result<Vec<Event>, DatabaseError>;

    /// Get `negentropy` items
    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        let events: Vec<Event> = self.query(vec![filter], Order::Desc).await?;
        Ok(events.into_iter().map(|e| (e.id, e.created_at)).collect())
    }

    /// Delete all events that match the [Filter]
    async fn delete(&self, filter: Filter) -> Result<(), DatabaseError>;

    /// Wipe all data
    async fn wipe(&self) -> Result<(), DatabaseError>;
}

/// Nostr Database Extension
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrDatabaseExt: NostrDatabase {
    /// Get profile metadata
    #[tracing::instrument(skip_all, level = "trace")]
    async fn profile(&self, public_key: PublicKey) -> Result<Profile, DatabaseError> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        let events: Vec<Event> = self.query(vec![filter], Order::Desc).await?;
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
    #[tracing::instrument(skip_all, level = "trace")]
    async fn contacts_public_keys(
        &self,
        public_key: PublicKey,
    ) -> Result<Vec<PublicKey>, DatabaseError> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        let events: Vec<Event> = self.query(vec![filter], Order::Desc).await?;
        match events.first() {
            Some(event) => Ok(event.public_keys().copied().collect()),
            None => Ok(Vec::new()),
        }
    }

    /// Get contact list with metadata of [`PublicKey`]
    #[tracing::instrument(skip_all, level = "trace")]
    async fn contacts(&self, public_key: PublicKey) -> Result<BTreeSet<Profile>, DatabaseError> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        let events: Vec<Event> = self.query(vec![filter], Order::Desc).await?;
        match events.first() {
            Some(event) => {
                // Get contacts metadata
                let filter = Filter::new()
                    .authors(event.public_keys().copied())
                    .kind(Kind::Metadata);
                let mut contacts: HashSet<Profile> = self
                    .query(vec![filter], Order::Desc)
                    .await?
                    .into_iter()
                    .map(|e| {
                        let metadata: Metadata =
                            Metadata::from_json(&e.content).unwrap_or_default();
                        Profile::new(e.pubkey, metadata)
                    })
                    .collect();

                // Extend with missing public keys
                contacts.extend(event.public_keys().copied().map(Profile::from));

                Ok(contacts.into_iter().collect())
            }
            None => Ok(BTreeSet::new()),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T: NostrDatabase + ?Sized> NostrDatabaseExt for T {}
