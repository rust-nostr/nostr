// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use nostr::prelude::*;

pub mod helper;

use crate::{DatabaseError, Events, Profile};

/// NIP65 relays map
pub type RelaysMap = HashMap<RelayUrl, Option<RelayMetadata>>;

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
pub trait NostrEventsDatabase: fmt::Debug + Send + Sync {
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

    // TODO: rename to `check_coordinate`?
    /// Check if [`Coordinate`] has been deleted before a certain [`Timestamp`]
    fn has_coordinate_been_deleted<'a>(
        &'a self,
        coordinate: &'a CoordinateBorrow<'a>,
        timestamp: &'a Timestamp,
    ) -> BoxedFuture<'a, Result<bool, DatabaseError>>;

    /// Set [`EventId`] as seen by relay
    ///
    /// Useful for NIP65 (aka gossip)
    #[deprecated(since = "0.39.0")]
    fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: RelayUrl,
    ) -> BoxedFuture<Result<(), DatabaseError>>;

    /// Get list of relays that have seen the [`EventId`]
    #[deprecated(
        since = "0.39.0",
        note = "For now this method hasn't a replacement and maybe will never have one. \
        You can keep track of seen IDs by looking at the Relay, RelayPool or Client notifications."
    )]
    fn event_seen_on_relays<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<HashSet<RelayUrl>>, DatabaseError>>;

    /// Get [`Event`] by [`EventId`]
    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>>;

    /// Count the number of events found with [`Filter`].
    ///
    /// Use `Filter::new()` or `Filter::default()` to count all events.
    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>>;

    /// Query stored events.
    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>>;

    /// Get `negentropy` items
    fn negentropy_items(
        &self,
        filter: Filter,
    ) -> BoxedFuture<Result<Vec<(EventId, Timestamp)>, DatabaseError>> {
        Box::pin(async move {
            let events: Events = self.query(filter).await?;
            Ok(events.into_iter().map(|e| (e.id, e.created_at)).collect())
        })
    }

    /// Delete all events that match the [Filter]
    fn delete(&self, filter: Filter) -> BoxedFuture<Result<(), DatabaseError>>;
}

/// Nostr Event Store Extension
pub trait NostrEventsDatabaseExt: NostrEventsDatabase {
    /// Get public key metadata
    fn metadata(
        &self,
        public_key: PublicKey,
    ) -> BoxedFuture<Result<Option<Metadata>, DatabaseError>> {
        Box::pin(async move {
            let filter = Filter::new()
                .author(public_key)
                .kind(Kind::Metadata)
                .limit(1);
            let events: Events = self.query(filter).await?;
            match events.first_owned() {
                Some(event) => Ok(Some(
                    Metadata::from_json(event.content).map_err(DatabaseError::backend)?,
                )),
                None => Ok(None),
            }
        })
    }

    /// Get contact list public keys
    fn contacts_public_keys(
        &self,
        public_key: PublicKey,
    ) -> BoxedFuture<Result<HashSet<PublicKey>, DatabaseError>> {
        Box::pin(async move {
            let filter = Filter::new()
                .author(public_key)
                .kind(Kind::ContactList)
                .limit(1);
            let events: Events = self.query(filter).await?;
            match events.first_owned() {
                Some(event) => Ok(event.tags.public_keys().copied().collect()),
                None => Ok(HashSet::new()),
            }
        })
    }

    /// Get contact list with metadata of [`PublicKey`]
    fn contacts(
        &self,
        public_key: PublicKey,
    ) -> BoxedFuture<Result<BTreeSet<Profile>, DatabaseError>> {
        Box::pin(async move {
            let filter = Filter::new()
                .author(public_key)
                .kind(Kind::ContactList)
                .limit(1);
            let events: Events = self.query(filter).await?;
            match events.first_owned() {
                Some(event) => {
                    // Get contacts metadata
                    let filter = Filter::new()
                        .authors(event.tags.public_keys().copied())
                        .kind(Kind::Metadata);
                    let mut contacts: HashSet<Profile> = self
                        .query(filter)
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
        })
    }

    /// Get relays list for [PublicKey]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    fn relay_list(&self, public_key: PublicKey) -> BoxedFuture<Result<RelaysMap, DatabaseError>> {
        Box::pin(async move {
            // Query
            let filter: Filter = Filter::default()
                .author(public_key)
                .kind(Kind::RelayList)
                .limit(1);
            let events: Events = self.query(filter).await?;

            // Extract relay list (NIP65)
            match events.first_owned() {
                Some(event) => Ok(nip65::extract_owned_relay_list(event).collect()),
                None => Ok(HashMap::new()),
            }
        })
    }

    /// Get relays list for public keys
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    fn relay_lists<'a, I>(
        &'a self,
        public_keys: I,
    ) -> BoxedFuture<'a, Result<HashMap<PublicKey, RelaysMap>, DatabaseError>>
    where
        I: IntoIterator<Item = PublicKey> + Send + 'a,
    {
        Box::pin(async move {
            // Query
            let filter: Filter = Filter::default().authors(public_keys).kind(Kind::RelayList);
            let events: Events = self.query(filter).await?;

            let mut map = HashMap::with_capacity(events.len());

            for event in events.into_iter() {
                map.insert(
                    event.pubkey,
                    nip65::extract_owned_relay_list(event).collect(),
                );
            }

            Ok(map)
        })
    }
}

impl<T: NostrEventsDatabase + ?Sized> NostrEventsDatabaseExt for T {}
