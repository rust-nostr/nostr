// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
use std::iter;
use std::ops::Bound;
use std::path::Path;

use heed::byteorder::NativeEndian;
use heed::types::{Bytes, Unit, U64};
use heed::{Database, Env, EnvFlags, EnvOpenOptions, RoRange, RoTxn, RwTxn};
use nostr::prelude::*;
use nostr_database::flatbuffers::FlatBufferDecodeBorrowed;
use nostr_database::{FlatBufferBuilder, FlatBufferEncode};

mod index;

use super::error::Error;
use super::types::{DatabaseEvent, DatabaseFilter};

const EVENT_ID_ALL_ZEROS: [u8; 32] = [0; 32];
const EVENT_ID_ALL_255: [u8; 32] = [255; 32];

// 64-bit
#[cfg(target_pointer_width = "64")]
const MAP_SIZE: usize = 1024 * 1024 * 1024 * 32; // 32GB

// 32-bit
#[cfg(target_pointer_width = "32")]
const MAP_SIZE: usize = 0xFFFFF000; // 4GB (2^32-4096)

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
    pub(crate) fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        // Construct LMDB env
        let env: Env = unsafe {
            EnvOpenOptions::new()
                .flags(EnvFlags::NO_TLS)
                .max_dbs(9)
                .map_size(MAP_SIZE)
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
    #[inline]
    pub(crate) fn read_txn(&self) -> Result<RoTxn, Error> {
        Ok(self.env.read_txn()?)
    }

    /// Get a write transaction
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
            &event.pubkey.to_bytes(),
            event.kind.as_u16(),
            &event.created_at,
            event.id.as_bytes(),
        );
        self.akc_index.put(txn, &akc_index_key, id)?;

        // Index by author (with created_at and id)
        let ac_index_key: Vec<u8> = index::make_ac_index_key(
            &event.pubkey.to_bytes(),
            &event.created_at,
            event.id.as_bytes(),
        );
        self.ac_index.put(txn, &ac_index_key, id)?;

        for tag in event.tags.iter() {
            if let (Some(tag_name), Some(tag_value)) = (tag.single_letter_tag(), tag.content()) {
                // Index by author and tag (with created_at and id)
                let atc_index_key: Vec<u8> = index::make_atc_index_key(
                    &event.pubkey.to_bytes(),
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
    pub(crate) fn remove(&self, txn: &mut RwTxn, event: &DatabaseEvent) -> Result<(), Error> {
        self.events.delete(txn, event.id())?;

        let ci_index_key: Vec<u8> = index::make_ci_index_key(&event.created_at, event.id());
        self.ci_index.delete(txn, &ci_index_key)?;

        let akc_index_key: Vec<u8> =
            index::make_akc_index_key(event.author(), event.kind, &event.created_at, event.id());
        self.akc_index.delete(txn, &akc_index_key)?;

        let ac_index_key: Vec<u8> =
            index::make_ac_index_key(event.author(), &event.created_at, event.id());
        self.ac_index.delete(txn, &ac_index_key)?;

        for tag in event.iter_tags() {
            if let Some((tag_name, tag_value)) = tag.extract() {
                // Index by author and tag (with created_at and id)
                let atc_index_key: Vec<u8> = index::make_atc_index_key(
                    event.author(),
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id(),
                );
                self.atc_index.delete(txn, &atc_index_key)?;

                // Index by kind and tag (with created_at and id)
                let ktc_index_key: Vec<u8> = index::make_ktc_index_key(
                    event.kind,
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id(),
                );
                self.ktc_index.delete(txn, &ktc_index_key)?;

                // Index by tag (with created_at and id)
                let tc_index_key: Vec<u8> =
                    index::make_tc_index_key(&tag_name, tag_value, &event.created_at, event.id());
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
    ) -> Result<Option<DatabaseEvent<'a>>, Error> {
        match self.events.get(txn, event_id)? {
            Some(bytes) => Ok(Some(DatabaseEvent::decode(bytes)?)),
            None => Ok(None),
        }
    }

    pub fn query<'a, I>(
        &self,
        txn: &'a RoTxn,
        filters: I,
    ) -> Result<BTreeSet<DatabaseEvent<'a>>, Error>
    where
        I: IntoIterator<Item = Filter>,
    {
        let mut output: BTreeSet<DatabaseEvent<'a>> = BTreeSet::new();
        for filter in filters.into_iter() {
            let events = self.single_filter_query(txn, filter)?;
            output.extend(events);
        }
        Ok(output)
    }

    pub fn delete(&self, read_txn: &RoTxn, txn: &mut RwTxn, filter: Filter) -> Result<(), Error> {
        let events = self.single_filter_query(read_txn, filter)?;
        for event in events.into_iter() {
            self.remove(txn, &event)?;
        }
        Ok(())
    }

    /// Find all events that match the filter
    fn single_filter_query<'a>(
        &self,
        txn: &'a RoTxn,
        filter: Filter,
    ) -> Result<Box<dyn Iterator<Item = DatabaseEvent<'a>> + 'a>, Error> {
        if let (Some(since), Some(until)) = (filter.since, filter.until) {
            if since > until {
                return Ok(Box::new(iter::empty()));
            }
        }

        // We insert into a BTreeSet to keep them time-ordered
        let mut output: BTreeSet<DatabaseEvent<'a>> = BTreeSet::new();

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

                if let Some(event) = self.get_event_by_id(txn, &id.0)? {
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
                    let iter = self.akc_iter(txn, &author.0, *kind, since, until)?;

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
                            self.atc_iter(txn, &author.0, tagname, tag_value, &since, &until)?;
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
                let iter = self.ac_iter(txn, &author.0, since, until)?;
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
        output: &mut BTreeSet<DatabaseEvent<'a>>,
    ) -> Result<(), Error> {
        let mut count: usize = 0;

        for result in iter {
            let (_key, value) = result?;

            // Get event by ID
            let event = self.get_event_by_id(txn, value)?.ok_or(Error::NotFound)?;

            if event.created_at < *since {
                break;
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
    ) -> Result<Option<DatabaseEvent<'a>>, Error> {
        if !kind.is_replaceable() {
            return Err(Error::WrongEventKind);
        }

        let mut iter = self.akc_iter(
            txn,
            &author.to_bytes(),
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
    ) -> Result<Option<DatabaseEvent<'a>>, Error> {
        if !addr.kind.is_addressable() {
            return Err(Error::WrongEventKind);
        }

        let iter = self.atc_iter(
            txn,
            &addr.public_key.to_bytes(),
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

    // Remove all replaceable events with the matching author-kind
    // Kind must be a replaceable (not parameterized replaceable) event kind
    pub fn remove_replaceable(
        &self,
        read_txn: &RoTxn,
        txn: &mut RwTxn,
        coordinate: &Coordinate,
        until: Timestamp,
    ) -> Result<(), Error> {
        if !coordinate.kind.is_replaceable() {
            return Err(Error::WrongEventKind);
        }

        let iter = self.akc_iter(
            read_txn,
            &coordinate.public_key.to_bytes(),
            coordinate.kind.as_u16(),
            Timestamp::zero(),
            until,
        )?;

        for result in iter {
            let (_key, id) = result?;

            if let Some(event) = self.get_event_by_id(read_txn, id)? {
                self.remove(txn, &event)?;
            }
        }

        Ok(())
    }

    // Remove all parameterized-replaceable events with the matching author-kind-d
    // Kind must be a parameterized-replaceable event kind
    pub fn remove_addressable(
        &self,
        read_txn: &RoTxn,
        txn: &mut RwTxn,
        coordinate: &Coordinate,
        until: Timestamp,
    ) -> Result<(), Error> {
        if !coordinate.kind.is_addressable() {
            return Err(Error::WrongEventKind);
        }

        let iter = self.atc_iter(
            read_txn,
            &coordinate.public_key.to_bytes(),
            &SingleLetterTag::lowercase(Alphabet::D),
            &coordinate.identifier,
            &Timestamp::min(),
            &until,
        )?;

        for result in iter {
            let (_key, id) = result?;

            // Our index doesn't have Kind embedded, so we have to check it
            let event = self.get_event_by_id(read_txn, id)?.ok_or(Error::NotFound)?;

            if event.kind == coordinate.kind.as_u16() {
                self.remove(txn, &event)?;
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn is_deleted(&self, txn: &RoTxn, event_id: &EventId) -> Result<bool, Error> {
        Ok(self.deleted_ids.get(txn, event_id.as_bytes())?.is_some())
    }

    pub(crate) fn mark_deleted(&self, txn: &mut RwTxn, event_id: &EventId) -> Result<(), Error> {
        self.deleted_ids.put(txn, event_id.as_bytes(), &())?;
        Ok(())
    }

    pub(crate) fn mark_coordinate_deleted(
        &self,
        txn: &mut RwTxn,
        coordinate: &Coordinate,
        when: Timestamp,
    ) -> Result<(), Error> {
        let key: Vec<u8> = index::make_coordinate_index_key(coordinate);
        self.deleted_coordinates.put(txn, &key, &when.as_u64())?;
        Ok(())
    }

    pub(crate) fn when_is_coordinate_deleted(
        &self,
        txn: &RoTxn,
        coordinate: &Coordinate,
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
}
