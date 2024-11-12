// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

use core::fmt;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

pub use async_trait::async_trait;
pub use nostr;
use nostr::nips::nip01::Coordinate;
use nostr::nips::nip65::{self, RelayMetadata};
use nostr::{Event, EventId, Filter, JsonUtil, Kind, Metadata, PublicKey, Timestamp, Url};

mod error;
mod events;
#[cfg(feature = "flatbuf")]
pub mod flatbuffers;
pub mod helper;
pub mod memory;
pub mod prelude;
pub mod profile;
mod tree;
mod util;

pub use self::error::DatabaseError;
pub use self::events::Events;
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

impl Backend {
    /// Check if it's a persistent backend
    ///
    /// All values different from [`Backend::Memory`] are considered persistent
    pub fn is_persistent(&self) -> bool {
        !matches!(self, Self::Memory)
    }
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
    /// The event is saved into database
    Saved,
    /// The event is marked as deleted
    Deleted,
    /// The event not exists
    NotExistent,
}

/// Nostr Database
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrDatabase: fmt::Debug + Send + Sync {
    /// Name of the backend database used
    fn backend(&self) -> Backend;

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    ///
    /// **This method assume that [`Event`] was already verified**
    async fn save_event(&self, event: &Event) -> Result<bool, DatabaseError>;

    /// Check event status by ID
    ///
    /// Check if the event is saved, deleted or not existent.
    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError>;

    /// Check if [`Coordinate`] has been deleted before a certain [`Timestamp`]
    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: &Timestamp,
    ) -> Result<bool, DatabaseError>;

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
    async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Event>, DatabaseError>;

    /// Count number of [`Event`] found by filters
    ///
    /// Use `Filter::new()` or `Filter::default()` to count all events.
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError>;

    /// Query store with filters
    async fn query(&self, filters: Vec<Filter>) -> Result<Events, DatabaseError>;

    /// Get `negentropy` items
    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        let events: Events = self.query(vec![filter]).await?;
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
        let events: Events = self.query(vec![filter]).await?;
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
        let events: Events = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => Ok(event.tags.public_keys().copied().collect()),
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
        let events: Events = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => {
                // Get contacts metadata
                let filter = Filter::new()
                    .authors(event.tags.public_keys().copied())
                    .kind(Kind::Metadata);
                let mut contacts: HashSet<Profile> = self
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
                contacts.extend(event.tags.public_keys().copied().map(Profile::from));

                Ok(contacts.into_iter().collect())
            }
            None => Ok(BTreeSet::new()),
        }
    }

    /// Get relays list for [PublicKey]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[tracing::instrument(skip_all, level = "trace")]
    async fn relay_list(
        &self,
        public_key: PublicKey,
    ) -> Result<HashMap<Url, Option<RelayMetadata>>, DatabaseError> {
        // Query
        let filter: Filter = Filter::default()
            .author(public_key)
            .kind(Kind::RelayList)
            .limit(1);
        let events: Events = self.query(vec![filter]).await?;

        // Extract relay list (NIP65)
        match events.first() {
            Some(event) => Ok(nip65::extract_relay_list(event)
                .map(|(u, m)| (u.clone(), *m))
                .collect()),
            None => Ok(HashMap::new()),
        }
    }

    /// Get relays list for public keys
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[tracing::instrument(skip_all, level = "trace")]
    async fn relay_lists<I>(
        &self,
        public_keys: I,
    ) -> Result<HashMap<PublicKey, HashMap<Url, Option<RelayMetadata>>>, DatabaseError>
    where
        I: IntoIterator<Item = PublicKey> + Send,
    {
        // Query
        let filter: Filter = Filter::default().authors(public_keys).kind(Kind::RelayList);
        let events: Events = self.query(vec![filter]).await?;

        let mut map = HashMap::with_capacity(events.len());

        for event in events.into_iter() {
            map.insert(
                event.pubkey,
                nip65::extract_owned_relay_list(event).collect(),
            );
        }

        Ok(map)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T: NostrDatabase + ?Sized> NostrDatabaseExt for T {}

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
