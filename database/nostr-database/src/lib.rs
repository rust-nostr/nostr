// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![cfg_attr(bench, feature(test))]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

#[cfg(bench)]
extern crate test;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub use nostr;
use nostr::prelude::*;

mod collections;
mod error;
pub mod ext;
#[cfg(feature = "flatbuf")]
pub mod flatbuffers;
pub mod prelude;
pub mod profile;

pub use self::collections::events::Events;
pub use self::error::DatabaseError;
#[cfg(feature = "flatbuf")]
pub use self::flatbuffers::{FlatBufferBuilder, FlatBufferDecode, FlatBufferEncode};
pub use self::profile::Profile;

/// NIP65 relays map
pub type RelaysMap = HashMap<RelayUrl, Option<RelayMetadata>>;

/// Backend type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backend {
    /// In-memory (RAM)
    Memory,
    /// Lightning Memory-Mapped Database
    LMDB,
    /// SQLite
    SQLite,
    /// Postgres
    Postgres,
    /// MySQL (i.e., MariaDB)
    MySql,
    /// RocksDB
    RocksDB,
    /// IndexedDB
    IndexedDB,
    /// MongoDB
    MongoDB,
    /// Redis
    Redis,
    /// Apache Cassandra
    Cassandra,
    /// Custom
    Custom(String),
}

impl Backend {
    /// Custom backend type
    #[inline]
    pub fn custom<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Self::Custom(name.into())
    }
}

/// Backend features
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Features {
    /// Whether the database supports persistent storage.
    pub persistent: bool,
    /// Whether the database supports event expiration (NIP-40)
    ///
    /// When supported, the database will automatically exclude expired events
    /// from query results and/or delete them.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    pub event_expiration: bool,
    /// Whether the database supports full-text search (NIP-50)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/50.md>
    pub full_text_search: bool,
    /// Whether the database supports the request to vanish (NIP-62)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/62.md>
    pub request_to_vanish: bool,
}

/// Database event status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DatabaseEventStatus {
    /// The event is saved into the database
    Saved,
    /// The event is marked as deleted
    Deleted,
    /// The event doesn't exist
    NotExistent,
}

/// Reason why event wasn't stored into the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RejectedReason {
    /// Ephemeral events aren't expected to be stored
    Ephemeral,
    /// The event already exists
    Duplicate,
    /// The event was deleted
    Deleted,
    /// The event is expired
    Expired,
    /// The event was replaced
    Replaced,
    /// Attempt to delete a non-owned event
    InvalidDelete,
    /// The event author vanished before
    Vanished,
    /// Other reason
    Other,
}

/// Save event status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SaveEventStatus {
    /// The event has been successfully saved
    Success,
    /// The event has been rejected
    Rejected(RejectedReason),
}

impl SaveEventStatus {
    /// Check if event is successfully saved
    #[inline]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
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

/// Nostr (Events) Database
pub trait NostrDatabase: Any + Debug + Send + Sync {
    /// Name of the backend database used
    fn backend(&self) -> Backend;

    /// Get backend features
    fn features(&self) -> Features;

    /// Save [`Event`] into store
    ///
    /// **This method assumes that [`Event`] was already verified**
    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>>;

    /// Check event status by ID
    ///
    /// Check if the event is saved, deleted or not existent.
    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>>;

    /// Get [`Event`] by [`EventId`]
    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>>;

    /// Count the number of events found with [`Filter`].
    ///
    /// Use `Filter::new()` or `Filter::default()` to count all events.
    fn count(&self, filter: Filter) -> BoxedFuture<'_, Result<usize, DatabaseError>>;

    /// Query stored events.
    fn query(&self, filter: Filter) -> BoxedFuture<'_, Result<Events, DatabaseError>>;

    /// Get `negentropy` items
    fn negentropy_items(
        &self,
        filter: Filter,
    ) -> BoxedFuture<'_, Result<Vec<(EventId, Timestamp)>, DatabaseError>> {
        Box::pin(async move {
            let events: Events = self.query(filter).await?;
            Ok(events.into_iter().map(|e| (e.id, e.created_at)).collect())
        })
    }

    /// Delete all events that match the [Filter]
    fn delete(&self, filter: Filter) -> BoxedFuture<'_, Result<(), DatabaseError>>;

    /// Wipe all data
    fn wipe(&self) -> BoxedFuture<'_, Result<(), DatabaseError>>;
}
