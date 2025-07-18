// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
use std::iter;
use std::ops::Bound;
use std::path::Path;

use heed::byteorder::NativeEndian;
use heed::types::{Bytes, Unit, U64};
use heed::{Database, Env, EnvFlags, EnvOpenOptions, RoRange, RoTxn, RwTxn};
use nostr::event::tag::TagStandard;
use nostr::nips::nip01::{Coordinate, CoordinateBorrow};
use nostr::prelude::*;
use nostr_database::flatbuffers::FlatBufferDecodeBorrowed;
use nostr_database::{FlatBufferBuilder, FlatBufferEncode, RejectedReason, SaveEventStatus};

mod index;

use super::error::Error;
use super::types::DatabaseFilter;

const EVENT_ID_ALL_ZEROS: [u8; 32] = [0; 32];
const EVENT_ID_ALL_255: [u8; 32] = [255; 32];

#[derive(Debug, Clone)]
pub(crate) struct Lmdb {
    /// LMDB env
    env: Env,
    /// Events
    events: Database<Bytes, Bytes>, // Event ID, Event
    /// CreatedAt + ID index
    ci_index: Database<Bytes, Bytes>, // <Index>, Event ID
    /// Tag + CreatedAt + ID index
    tc_index: Database<Bytes, Bytes>, // <Index>, Event ID
    /// Author + CreatedAt + ID index
    ac_index: Database<Bytes, Bytes>, // <Index>, Event ID
    /// Author + Kind + CreatedAt + ID index
    akc_index: Database<Bytes, Bytes>, // <Index>, Event ID
    /// Author + Tag + CreatedAt + ID index
    atc_index: Database<Bytes, Bytes>, // <Index>, Event ID
    /// Kind + Tag + CreatedAt + ID index
    ktc_index: Database<Bytes, Bytes>, // <Index>, Event ID
    /// Deleted IDs
    deleted_ids: Database<Bytes, Unit>, // Event ID
    /// Deleted coordinates
    deleted_coordinates: Database<Bytes, U64<NativeEndian>>, // Coordinate, UNIX timestamp
}

impl Lmdb {
    pub(super) fn new<P>(
        path: P,
        map_size: usize,
        max_readers: u32,
        max_dbs: u32,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        // Construct LMDB env
        let env: Env = unsafe {
            EnvOpenOptions::new()
                .flags(EnvFlags::NO_TLS)
                .max_dbs(max_dbs)
                .max_readers(max_readers)
                .map_size(map_size)
                .open(path)?
        };

        // Acquire write transaction
        let mut txn = env.write_txn()?;

        // Open/Create maps
        let events = env
            .database_options()
            .types::<Bytes, Bytes>()
            .create(&mut txn)?;
        let ci_index = env
            .database_options()
            .types::<Bytes, Bytes>()
            .name("ci")
            .create(&mut txn)?;
        let tc_index = env
            .database_options()
            .types::<Bytes, Bytes>()
            .name("tci")
            .create(&mut txn)?;
        let ac_index = env
            .database_options()
            .types::<Bytes, Bytes>()
            .name("aci")
            .create(&mut txn)?;
        let akc_index = env
            .database_options()
            .types::<Bytes, Bytes>()
            .name("akci")
            .create(&mut txn)?;
        let atc_index = env
            .database_options()
            .types::<Bytes, Bytes>()
            .name("atci")
            .create(&mut txn)?;
        let ktc_index = env
            .database_options()
            .types::<Bytes, Bytes>()
            .name("ktci")
            .create(&mut txn)?;
        let deleted_ids = env
            .database_options()
            .types::<Bytes, Unit>()
            .name("deleted-ids")
            .create(&mut txn)?;
        let deleted_coordinates = env
            .database_options()
            .types::<Bytes, U64<NativeEndian>>()
            .name("deleted-coordinates")
            .create(&mut txn)?;

        // Commit changes
        txn.commit()?;

        Ok(Self {
            env,
            events,
            ci_index,
            tc_index,
            ac_index,
            akc_index,
            atc_index,
            ktc_index,
            deleted_ids,
            deleted_coordinates,
        })
    }

    /// Get a read transaction
    ///
    /// This should never block the current thread
    #[inline]
    pub(crate) fn read_txn(&self) -> Result<RoTxn, Error> {
        Ok(self.env.read_txn()?)
    }

    /// Get a write transaction
    ///
    /// This blocks the current thread if there is another write txn
    #[inline]
    pub(crate) fn write_txn(&self) -> Result<RwTxn, Error> {
        Ok(self.env.write_txn()?)
    }

    /// Store and index the event
    pub(crate) fn store(
        &self,
        txn: &mut RwTxn,
        fbb: &mut FlatBufferBuilder,
        event: &Event,
    ) -> Result<(), Error> {
        let id: &[u8] = event.id.as_bytes();

        // Store event
        self.events.put(txn, id, event.encode(fbb))?;

        // Index by created_at and id
        let ci_index_key: Vec<u8> =
            index::make_ci_index_key(&event.created_at, event.id.as_bytes());
        self.ci_index.put(txn, &ci_index_key, id)?;

        // Index by author and kind (with created_at and id)
        let akc_index_key: Vec<u8> = index::make_akc_index_key(
            event.pubkey.as_bytes(),
            event.kind.as_u16(),
            &event.created_at,
            event.id.as_bytes(),
        );
        self.akc_index.put(txn, &akc_index_key, id)?;

        // Index by author (with created_at and id)
        let ac_index_key: Vec<u8> = index::make_ac_index_key(
            event.pubkey.as_bytes(),
            &event.created_at,
            event.id.as_bytes(),
        );
        self.ac_index.put(txn, &ac_index_key, id)?;

        for tag in event.tags.iter() {
            if let (Some(tag_name), Some(tag_value)) = (tag.single_letter_tag(), tag.content()) {
                // Index by author and tag (with created_at and id)
                let atc_index_key: Vec<u8> = index::make_atc_index_key(
                    event.pubkey.as_bytes(),
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id.as_bytes(),
                );
                self.atc_index.put(txn, &atc_index_key, id)?;

                // Index by kind and tag (with created_at and id)
                let ktc_index_key: Vec<u8> = index::make_ktc_index_key(
                    event.kind.as_u16(),
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id.as_bytes(),
                );
                self.ktc_index.put(txn, &ktc_index_key, id)?;

                // Index by tag (with created_at and id)
                let tc_index_key: Vec<u8> = index::make_tc_index_key(
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id.as_bytes(),
                );
                self.tc_index.put(txn, &tc_index_key, id)?;
            }
        }

        Ok(())
    }

    /// Remove the event
    pub(crate) fn remove(&self, txn: &mut RwTxn, event: &EventBorrow) -> Result<(), Error> {
        self.events.delete(txn, event.id)?;

        let ci_index_key: Vec<u8> = index::make_ci_index_key(&event.created_at, event.id);
        self.ci_index.delete(txn, &ci_index_key)?;

        let akc_index_key: Vec<u8> =
            index::make_akc_index_key(event.pubkey, event.kind, &event.created_at, event.id);
        self.akc_index.delete(txn, &akc_index_key)?;

        let ac_index_key: Vec<u8> =
            index::make_ac_index_key(event.pubkey, &event.created_at, event.id);
        self.ac_index.delete(txn, &ac_index_key)?;

        for tag in event.tags.iter() {
            if let Some((tag_name, tag_value)) = tag.extract() {
                // Index by author and tag (with created_at and id)
                let atc_index_key: Vec<u8> = index::make_atc_index_key(
                    event.pubkey,
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id,
                );
                self.atc_index.delete(txn, &atc_index_key)?;

                // Index by kind and tag (with created_at and id)
                let ktc_index_key: Vec<u8> = index::make_ktc_index_key(
                    event.kind,
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id,
                );
                self.ktc_index.delete(txn, &ktc_index_key)?;

                // Index by tag (with created_at and id)
                let tc_index_key: Vec<u8> =
                    index::make_tc_index_key(&tag_name, tag_value, &event.created_at, event.id);
                self.tc_index.delete(txn, &tc_index_key)?;
            }
        }

        Ok(())
    }

    pub(crate) fn wipe(&self, txn: &mut RwTxn) -> Result<(), Error> {
        self.events.clear(txn)?;
        self.ci_index.clear(txn)?;
        self.tc_index.clear(txn)?;
        self.ac_index.clear(txn)?;
        self.akc_index.clear(txn)?;
        self.atc_index.clear(txn)?;
        self.ktc_index.clear(txn)?;
        self.deleted_ids.clear(txn)?;
        self.deleted_coordinates.clear(txn)?;
        Ok(())
    }

    #[inline]
    pub(crate) fn has_event(&self, txn: &RoTxn, event_id: &[u8; 32]) -> Result<bool, Error> {
        Ok(self.get_event_by_id(txn, event_id)?.is_some())
    }

    #[inline]
    pub(crate) fn get_event_by_id<'a>(
        &self,
        txn: &'a RoTxn,
        event_id: &[u8],
    ) -> Result<Option<EventBorrow<'a>>, Error> {
        match self.events.get(txn, event_id)? {
            Some(bytes) => Ok(Some(EventBorrow::decode(bytes)?)),
            None => Ok(None),
        }
    }

    /// Find all events that match the filter
    pub fn query<'a>(
        &self,
        txn: &'a RoTxn,
        filter: Filter,
    ) -> Result<Box<dyn Iterator<Item = EventBorrow<'a>> + 'a>, Error> {
        if let (Some(since), Some(until)) = (filter.since, filter.until) {
            if since > until {
                return Ok(Box::new(iter::empty()));
            }
        }

        // We insert into a BTreeSet to keep them time-ordered
        let mut output: BTreeSet<EventBorrow<'a>> = BTreeSet::new();

        let limit: Option<usize> = filter.limit;
        let since = filter.since.unwrap_or_else(Timestamp::min);
        let until = filter.until.unwrap_or_else(Timestamp::max);

        let filter: DatabaseFilter = filter.into();

        if !filter.ids.is_empty() {
            // Fetch by id
            for id in filter.ids.iter() {
                // Check if limit is set
                if let Some(limit) = limit {
                    // Stop if limited
                    if output.len() >= limit {
                        break;
                    }
                }

                if let Some(event) = self.get_event_by_id(txn, id)? {
                    // Skip deleted events
                    if self.is_deleted(txn, event.id)? {
                        continue;
                    }

                    if filter.match_event(&event) {
                        output.insert(event);
                    }
                }
            }
        } else if !filter.authors.is_empty() && !filter.kinds.is_empty() {
            // We may bring since forward if we hit the limit without going back that
            // far, so we use a mutable since:
            let mut since = since;

            for author in filter.authors.iter() {
                for kind in filter.kinds.iter() {
                    let iter = self.akc_iter(txn, author, *kind, since, until)?;

                    // Count how many we have found of this author-kind pair, so we
                    // can possibly update `since`
                    let mut paircount = 0;

                    'per_event: for result in iter {
                        let (_key, value) = result?;
                        let event = self.get_event_by_id(txn, value)?.ok_or(Error::NotFound)?;

                        // If we have gone beyond since, we can stop early
                        // (We have to check because `since` might change in this loop)
                        if event.created_at < since {
                            break 'per_event;
                        }

                        // Skip deleted events
                        if self.is_deleted(txn, event.id)? {
                            continue 'per_event;
                        }

                        // check against the rest of the filter
                        if filter.match_event(&event) {
                            let created_at = event.created_at;

                            // Accept the event
                            output.insert(event);
                            paircount += 1;

                            // Stop this pair if limited
                            if let Some(limit) = limit {
                                if paircount >= limit {
                                    // Since we found the limit just among this pair,
                                    // potentially move since forward
                                    if created_at > since {
                                        since = created_at;
                                    }
                                    break 'per_event;
                                }
                            }

                            // If kind is replaceable (and not parameterized)
                            // then don't take any more events for this author-kind
                            // pair.
                            // NOTE that this optimization is difficult to implement
                            // for other replaceable event situations
                            if Kind::from(*kind).is_replaceable() {
                                break 'per_event;
                            }
                        }
                    }
                }
            }
        } else if !filter.authors.is_empty() && !filter.generic_tags.is_empty() {
            // We may bring since forward if we hit the limit without going back that
            // far, so we use a mutable since:
            let mut since = since;

            for author in filter.authors.iter() {
                for (tagname, set) in filter.generic_tags.iter() {
                    for tag_value in set.iter() {
                        let iter =
                            self.atc_iter(txn, author, tagname, tag_value, &since, &until)?;
                        self.iterate_filter_until_limit(
                            txn,
                            &filter,
                            iter,
                            &mut since,
                            limit,
                            &mut output,
                        )?;
                    }
                }
            }
        } else if !filter.kinds.is_empty() && !filter.generic_tags.is_empty() {
            // We may bring since forward if we hit the limit without going back that
            // far, so we use a mutable since:
            let mut since = since;

            for kind in filter.kinds.iter() {
                for (tag_name, set) in filter.generic_tags.iter() {
                    for tag_value in set.iter() {
                        let iter =
                            self.ktc_iter(txn, *kind, tag_name, tag_value, &since, &until)?;
                        self.iterate_filter_until_limit(
                            txn,
                            &filter,
                            iter,
                            &mut since,
                            limit,
                            &mut output,
                        )?;
                    }
                }
            }
        } else if !filter.generic_tags.is_empty() {
            // We may bring since forward if we hit the limit without going back that
            // far, so we use a mutable since:
            let mut since = since;

            for (tag_name, set) in filter.generic_tags.iter() {
                for tag_value in set.iter() {
                    let iter = self.tc_iter(txn, tag_name, tag_value, &since, &until)?;
                    self.iterate_filter_until_limit(
                        txn,
                        &filter,
                        iter,
                        &mut since,
                        limit,
                        &mut output,
                    )?;
                }
            }
        } else if !filter.authors.is_empty() {
            // We may bring since forward if we hit the limit without going back that
            // far, so we use a mutable since:
            let mut since = since;

            for author in filter.authors.iter() {
                let iter = self.ac_iter(txn, author, since, until)?;
                self.iterate_filter_until_limit(
                    txn,
                    &filter,
                    iter,
                    &mut since,
                    limit,
                    &mut output,
                )?;
            }
        } else {
            // SCRAPE
            // This is INEFFICIENT as it scans through many events

            let iter = self.ci_iter(txn, &since, &until)?;
            for result in iter {
                // Check if limit is set
                if let Some(limit) = limit {
                    // Stop if limited
                    if output.len() >= limit {
                        break;
                    }
                }

                let (_key, value) = result?;
                let event = self.get_event_by_id(txn, value)?.ok_or(Error::NotFound)?;

                // Skip deleted events
                if self.is_deleted(txn, event.id)? {
                    continue;
                }

                if filter.match_event(&event) {
                    output.insert(event);
                }
            }
        }

        // Optionally apply limit
        Ok(match limit {
            Some(limit) => Box::new(output.into_iter().take(limit)),
            None => Box::new(output.into_iter()),
        })
    }

    fn iterate_filter_until_limit<'a>(
        &self,
        txn: &'a RoTxn,
        filter: &DatabaseFilter,
        iter: RoRange<Bytes, Bytes>,
        since: &mut Timestamp,
        limit: Option<usize>,
        output: &mut BTreeSet<EventBorrow<'a>>,
    ) -> Result<(), Error> {
        let mut count: usize = 0;

        for result in iter {
            let (_key, value) = result?;

            // Get event by ID
            let event = self.get_event_by_id(txn, value)?.ok_or(Error::NotFound)?;

            if event.created_at < *since {
                break;
            }

            // Skip deleted events
            if self.is_deleted(txn, event.id)? {
                continue;
            }

            // check against the rest of the filter
            if filter.match_event(&event) {
                let created_at = event.created_at;

                // Accept the event
                output.insert(event);
                count += 1;

                // Check if limit is set
                if let Some(limit) = limit {
                    // Stop this limited
                    if count >= limit {
                        if created_at > *since {
                            *since = created_at;
                        }
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn find_replaceable_event<'a>(
        &self,
        txn: &'a RoTxn,
        author: &PublicKey,
        kind: Kind,
    ) -> Result<Option<EventBorrow<'a>>, Error> {
        if !kind.is_replaceable() {
            return Err(Error::WrongEventKind);
        }

        let mut iter = self.akc_iter(
            txn,
            author.as_bytes(),
            kind.as_u16(),
            Timestamp::min(),
            Timestamp::max(),
        )?;

        if let Some(result) = iter.next() {
            let (_key, id) = result?;
            return self.get_event_by_id(txn, id);
        }

        Ok(None)
    }

    pub fn find_addressable_event<'a>(
        &'a self,
        txn: &'a RoTxn,
        addr: &Coordinate,
    ) -> Result<Option<EventBorrow<'a>>, Error> {
        if !addr.kind.is_addressable() {
            return Err(Error::WrongEventKind);
        }

        let iter = self.atc_iter(
            txn,
            addr.public_key.as_bytes(),
            &SingleLetterTag::lowercase(Alphabet::D),
            &addr.identifier,
            &Timestamp::min(),
            &Timestamp::max(),
        )?;

        for result in iter {
            let (_key, id) = result?;
            let event = self.get_event_by_id(txn, id)?.ok_or(Error::NotFound)?;

            // the atc index doesn't have kind, so we have to compare the kinds
            if event.kind != addr.kind.as_u16() {
                continue;
            }

            return Ok(Some(event));
        }

        Ok(None)
    }

    #[inline]
    pub(crate) fn is_deleted(&self, txn: &RoTxn, event_id_bytes: &[u8]) -> Result<bool, Error> {
        Ok(self.deleted_ids.get(txn, event_id_bytes)?.is_some())
    }

    pub(crate) fn mark_deleted(&self, txn: &mut RwTxn, event_id_bytes: &[u8]) -> Result<(), Error> {
        self.deleted_ids.put(txn, event_id_bytes, &())?;
        Ok(())
    }

    pub(crate) fn mark_coordinate_deleted(
        &self,
        txn: &mut RwTxn,
        coordinate: &CoordinateBorrow,
        when: Timestamp,
    ) -> Result<(), Error> {
        let key: Vec<u8> = index::make_coordinate_index_key(coordinate);
        self.deleted_coordinates.put(txn, &key, &when.as_u64())?;
        Ok(())
    }

    pub(crate) fn when_is_coordinate_deleted<'a>(
        &self,
        txn: &RoTxn,
        coordinate: &'a CoordinateBorrow<'a>,
    ) -> Result<Option<Timestamp>, Error> {
        let key: Vec<u8> = index::make_coordinate_index_key(coordinate);
        Ok(self
            .deleted_coordinates
            .get(txn, &key)?
            .map(Timestamp::from_secs))
    }

    pub(crate) fn ci_iter<'a>(
        &'a self,
        txn: &'a RoTxn,
        since: &Timestamp,
        until: &Timestamp,
    ) -> Result<RoRange<'a, Bytes, Bytes>, Error> {
        let start_prefix = index::make_ci_index_key(until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_ci_index_key(since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        Ok(self.ci_index.range(txn, &range)?)
    }

    pub(crate) fn tc_iter<'a>(
        &'a self,
        txn: &'a RoTxn,
        tag_name: &SingleLetterTag,
        tag_value: &str,
        since: &Timestamp,
        until: &Timestamp,
    ) -> Result<RoRange<'a, Bytes, Bytes>, Error> {
        let start_prefix = index::make_tc_index_key(
            tag_name,
            tag_value,
            until, // scan goes backwards in time
            &EVENT_ID_ALL_ZEROS,
        );
        let end_prefix = index::make_tc_index_key(tag_name, tag_value, since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        Ok(self.tc_index.range(txn, &range)?)
    }

    pub(crate) fn ac_iter<'a>(
        &'a self,
        txn: &'a RoTxn,
        author: &[u8; 32],
        since: Timestamp,
        until: Timestamp,
    ) -> Result<RoRange<'a, Bytes, Bytes>, Error> {
        let start_prefix = index::make_ac_index_key(author, &until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_ac_index_key(author, &since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        Ok(self.ac_index.range(txn, &range)?)
    }

    pub(crate) fn akc_iter<'a>(
        &'a self,
        txn: &'a RoTxn,
        author: &[u8; 32],
        kind: u16,
        since: Timestamp,
        until: Timestamp,
    ) -> Result<RoRange<'a, Bytes, Bytes>, Error> {
        let start_prefix = index::make_akc_index_key(author, kind, &until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_akc_index_key(author, kind, &since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        Ok(self.akc_index.range(txn, &range)?)
    }

    pub(crate) fn atc_iter<'a>(
        &'a self,
        txn: &'a RoTxn,
        author: &[u8; 32],
        tag_name: &SingleLetterTag,
        tag_value: &str,
        since: &Timestamp,
        until: &Timestamp,
    ) -> Result<RoRange<'a, Bytes, Bytes>, Error> {
        let start_prefix: Vec<u8> = index::make_atc_index_key(
            author,
            tag_name,
            tag_value,
            until, // scan goes backwards in time
            &EVENT_ID_ALL_ZEROS,
        );
        let end_prefix: Vec<u8> =
            index::make_atc_index_key(author, tag_name, tag_value, since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        Ok(self.atc_index.range(txn, &range)?)
    }

    pub(crate) fn ktc_iter<'a>(
        &'a self,
        txn: &'a RoTxn,
        kind: u16,
        tag_name: &SingleLetterTag,
        tag_value: &str,
        since: &Timestamp,
        until: &Timestamp,
    ) -> Result<RoRange<'a, Bytes, Bytes>, Error> {
        let start_prefix = index::make_ktc_index_key(
            kind,
            tag_name,
            tag_value,
            until, // scan goes backwards in time
            &EVENT_ID_ALL_ZEROS,
        );
        let end_prefix =
            index::make_ktc_index_key(kind, tag_name, tag_value, since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        Ok(self.ktc_index.range(txn, &range)?)
    }

    // Transaction-aware save method for batched operations
    pub(crate) fn save_event_with_txn(
        &self,
        read_txn: &RoTxn,
        txn: &mut RwTxn,
        fbb: &mut FlatBufferBuilder,
        event: &Event,
    ) -> Result<SaveEventStatus, Error> {
        // Check if event is ephemeral
        if event.kind.is_ephemeral() {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Ephemeral));
        }

        // Check if event already exists
        if self.has_event(read_txn, event.id.as_bytes())? {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Duplicate));
        }

        // Check if event ID was deleted
        if self.is_deleted(read_txn, event.id.as_bytes())? {
            return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
        }

        // Check if coordinate was deleted after event's created_at date
        if let Some(coordinate) = event.coordinate() {
            let coord_borrow = CoordinateBorrow {
                kind: coordinate.kind,
                public_key: coordinate.public_key,
                identifier: coordinate.identifier,
            };
            if let Some(time) = self.when_is_coordinate_deleted(read_txn, &coord_borrow)? {
                if event.created_at <= time {
                    return Ok(SaveEventStatus::Rejected(RejectedReason::Deleted));
                }
            }
        }

        // Handle deletion events
        if event.kind == Kind::EventDeletion {
            for tag in event.tags.iter() {
                if let Some(tag_standard) = tag.as_standardized() {
                    match tag_standard {
                        TagStandard::Event {
                            event_id: event_id_to_delete,
                            ..
                        } => {
                            if let Some(target_event) =
                                self.get_event_by_id(read_txn, event_id_to_delete.as_bytes())?
                            {
                                if target_event.pubkey == event.pubkey.as_bytes() {
                                    // Only mark as deleted, don't remove from database
                                    self.mark_deleted(txn, event_id_to_delete.as_bytes())?;
                                } else {
                                    // Only allow deletion of own events
                                    return Ok(SaveEventStatus::Rejected(
                                        RejectedReason::InvalidDelete,
                                    ));
                                }
                            }
                        }
                        TagStandard::Coordinate {
                            coordinate: coord_to_delete,
                            ..
                        } => {
                            // Only allow deletion of own coordinates
                            if coord_to_delete.public_key != event.pubkey {
                                return Ok(SaveEventStatus::Rejected(
                                    RejectedReason::InvalidDelete,
                                ));
                            }

                            // Addressable events must have a non-empty identifier
                            if coord_to_delete.kind.is_addressable()
                                && coord_to_delete.identifier.is_empty()
                            {
                                return Ok(SaveEventStatus::Rejected(
                                    RejectedReason::InvalidDelete,
                                ));
                            }

                            // Mark coordinate as deleted
                            let coord_borrow = CoordinateBorrow {
                                kind: &coord_to_delete.kind,
                                public_key: &coord_to_delete.public_key,
                                identifier: Some(coord_to_delete.identifier.as_str()),
                            };
                            self.mark_coordinate_deleted(txn, &coord_borrow, event.created_at)?;

                            // Mark all events matching the coordinate as deleted
                            let iter = self.atc_iter(
                                read_txn,
                                coord_to_delete.public_key.as_bytes(),
                                &SingleLetterTag::lowercase(Alphabet::D),
                                &coord_to_delete.identifier,
                                &Timestamp::min(),
                                &Timestamp::max(),
                            )?;

                            for result in iter {
                                let (_key, id) = result?;
                                let event =
                                    self.get_event_by_id(read_txn, id)?.ok_or(Error::NotFound)?;

                                // the atc index doesn't have kind, so we have to compare the kinds
                                if event.kind == coord_to_delete.kind.as_u16() {
                                    self.mark_deleted(txn, event.id)?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Handle replaceable events
        if event.kind.is_replaceable() {
            if let Some(existing_event) =
                self.find_replaceable_event(read_txn, &event.pubkey, event.kind)?
            {
                if event.created_at < existing_event.created_at
                    || (event.created_at == existing_event.created_at
                        && event.id.as_bytes() < existing_event.id)
                {
                    return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                }
                // Remove the existing event as it's being replaced
                let event_id_bytes = existing_event.id;
                drop(existing_event);
                self.remove_event(txn, event_id_bytes)?;
            }
        }

        // Handle addressable (parameterized replaceable) events
        if event.kind.is_addressable() {
            if let Some(coordinate_borrow) = event.coordinate() {
                // Convert CoordinateBorrow to Coordinate
                let coordinate = Coordinate {
                    kind: *coordinate_borrow.kind,
                    public_key: *coordinate_borrow.public_key,
                    identifier: coordinate_borrow
                        .identifier
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                };
                if let Some(existing_event) = self.find_addressable_event(read_txn, &coordinate)? {
                    if event.created_at < existing_event.created_at
                        || (event.created_at == existing_event.created_at
                            && event.id.as_bytes() < existing_event.id)
                    {
                        return Ok(SaveEventStatus::Rejected(RejectedReason::Replaced));
                    }
                    // Convert to bytes and drop the borrow before removing
                    let event_id_bytes = existing_event.id;
                    drop(existing_event);
                    self.remove_event(txn, event_id_bytes)?;
                }
            }
        }

        // Store the event
        self.store(txn, fbb, event)?;

        Ok(SaveEventStatus::Success)
    }

    pub(crate) fn remove_event(&self, txn: &mut RwTxn, event_id_bytes: &[u8]) -> Result<(), Error> {
        // First get the raw event bytes
        let event_bytes = match self.events.get(txn, event_id_bytes)? {
            Some(bytes) => {
                // Copy the bytes to owned data
                let owned_bytes: Vec<u8> = bytes.to_vec();
                Some(owned_bytes)
            }
            None => None,
        };

        // Now we can work with the owned bytes
        if let Some(bytes) = event_bytes {
            // Decode and remove
            let event_borrow = EventBorrow::decode(&bytes)?;
            self.remove(txn, &event_borrow)?;
        }

        Ok(())
    }

    pub(crate) fn delete_with_txn(
        &self,
        read_txn: &RoTxn,
        txn: &mut RwTxn,
        filter: super::Filter,
    ) -> Result<usize, Error> {
        let events = self.query(read_txn, filter)?;
        let mut count = 0;

        // Delete each event
        for event in events {
            self.remove_event(txn, event.id)?;
            count += 1;
        }

        Ok(count)
    }
}
