// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use nostr::prelude::*;

pub mod helper;

use crate::{DatabaseError, Events, Profile};

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
pub trait IntoNostrEventsDatabase {
    fn into_database(self) -> Arc<dyn NostrEventsDatabase>;
}

impl IntoNostrEventsDatabase for Arc<dyn NostrEventsDatabase> {
    fn into_database(self) -> Arc<dyn NostrEventsDatabase> {
        self
    }
}

impl<T> IntoNostrEventsDatabase for T
where
    T: NostrEventsDatabase + Sized + 'static,
{
    fn into_database(self) -> Arc<dyn NostrEventsDatabase> {
        Arc::new(self)
    }
}

impl<T> IntoNostrEventsDatabase for Arc<T>
where
    T: NostrEventsDatabase + 'static,
{
    fn into_database(self) -> Arc<dyn NostrEventsDatabase> {
        self
    }
}

/// Nostr Events Database
///
/// Store for the nostr events.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrEventsDatabase: fmt::Debug + Send + Sync {
    /// Save [`Event`] into store
    ///
    /// **This method assumes that [`Event`] was already verified**
    async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, DatabaseError>;

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
    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: RelayUrl,
    ) -> Result<(), DatabaseError>;

    /// Get list of relays that have seen the [`EventId`]
    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError>;

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
}

/// Nostr Event Store Extension
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait NostrEventsDatabaseExt: NostrEventsDatabase {
    /// Get public key metadata
    async fn metadata(&self, public_key: PublicKey) -> Result<Option<Metadata>, DatabaseError> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        let events: Events = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => Ok(Some(
                Metadata::from_json(&event.content).map_err(DatabaseError::backend)?,
            )),
            None => Ok(None),
        }
    }

    /// Get contact list public keys
    async fn contacts_public_keys(
        &self,
        public_key: PublicKey,
    ) -> Result<HashSet<PublicKey>, DatabaseError> {
        let filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        let events: Events = self.query(vec![filter]).await?;
        match events.first() {
            Some(event) => Ok(event.tags.public_keys().copied().collect()),
            None => Ok(HashSet::new()),
        }
    }

    /// Get contact list with metadata of [`PublicKey`]
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
    async fn relay_list(
        &self,
        public_key: PublicKey,
    ) -> Result<HashMap<RelayUrl, Option<RelayMetadata>>, DatabaseError> {
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
    async fn relay_lists<I>(
        &self,
        public_keys: I,
    ) -> Result<HashMap<PublicKey, HashMap<RelayUrl, Option<RelayMetadata>>>, DatabaseError>
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
impl<T: NostrEventsDatabase + ?Sized> NostrEventsDatabaseExt for T {}
