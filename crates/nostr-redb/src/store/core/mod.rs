// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
use std::ops::Bound;
use std::path::Path;
use std::sync::Arc;
use std::{fs, iter};

use nostr::prelude::*;
use nostr_database::flatbuffers::FlatBufferDecodeBorrowed;
use nostr_database::{FlatBufferBuilder, FlatBufferEncode};
use redb::{Database, Range, ReadTransaction, TableDefinition, WriteTransaction};

pub(super) mod index;

use super::error::Error;
use super::types::{AccessGuardEvent, DatabaseFilter};

const EVENT_ID_ALL_ZEROS: [u8; 32] = [0; 32];
const EVENT_ID_ALL_255: [u8; 32] = [255; 32];

const EVENTS: TableDefinition<&[u8; 32], &[u8]> = TableDefinition::new("events"); // Event ID, Event
/// CreatedAt + ID index
const CI_INDEX: TableDefinition<&[u8], &[u8; 32]> = TableDefinition::new("ci_index"); // <index>, Event ID
/// Tag + CreatedAt + ID index
const TC_INDEX: TableDefinition<&[u8], &[u8; 32]> = TableDefinition::new("tc_index"); // <index>, Event ID
/// Author + CreatedAt + ID index
const AC_INDEX: TableDefinition<&[u8], &[u8; 32]> = TableDefinition::new("ac_index"); // <index>, Event ID
/// Author + Kind + CreatedAt + ID index
const AKC_INDEX: TableDefinition<&[u8], &[u8; 32]> = TableDefinition::new("akc_index"); // <index>, Event ID
/// Author + Tag + CreatedAt + ID index
const ATC_INDEX: TableDefinition<&[u8], &[u8; 32]> = TableDefinition::new("atc_index"); // <index>, Event ID
/// Kind + Tag + CreatedAt + ID index
const KTC_INDEX: TableDefinition<&[u8], &[u8; 32]> = TableDefinition::new("ktc_index"); // <index>, Event ID
const DELETED_IDS: TableDefinition<&[u8; 32], ()> = TableDefinition::new("deleted_ids"); // Event ID
const DELETED_COORDINATES: TableDefinition<&[u8], u64> =
    TableDefinition::new("deletec_coordinates"); // Coordinate, UNIX timestamp

type IndexRange<'a> = Range<'a, &'static [u8], &'static [u8; 32]>;

#[derive(Debug, Clone)]
pub(crate) struct Db {
    env: Arc<Database>,
}

impl Db {
    pub(crate) fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let dir = path.as_ref();

        fs::create_dir_all(dir)?;

        let path = dir.join("data.redb");
        let env = Arc::new(Database::create(path)?);

        // Create tables
        let txn = env.begin_write()?;
        Self::create_tables(&txn)?;
        txn.commit()?;

        Ok(Self { env })
    }

    /// Get a read transaction
    #[inline]
    pub(crate) fn read_txn(&self) -> Result<ReadTransaction, Error> {
        Ok(self.env.begin_read()?)
    }

    /// Get a write transaction
    #[inline]
    pub(crate) fn write_txn(&self) -> Result<WriteTransaction, Error> {
        Ok(self.env.begin_write()?)
    }

    fn create_tables(txn: &WriteTransaction) -> Result<(), Error> {
        txn.open_table(EVENTS)?;
        txn.open_table(CI_INDEX)?;
        txn.open_table(TC_INDEX)?;
        txn.open_table(AC_INDEX)?;
        txn.open_table(AKC_INDEX)?;
        txn.open_table(ATC_INDEX)?;
        txn.open_table(KTC_INDEX)?;
        txn.open_table(DELETED_IDS)?;
        txn.open_table(DELETED_COORDINATES)?;

        Ok(())
    }

    /// Store and index the event
    pub(crate) fn store(
        &self,
        txn: &WriteTransaction,
        fbb: &mut FlatBufferBuilder,
        event: &Event,
    ) -> Result<(), Error> {
        let id: &[u8; 32] = event.id.as_bytes();

        // Store event
        let mut events = txn.open_table(EVENTS)?;
        events.insert(id, event.encode(fbb))?;

        // Index by created_at and id
        let ci_index_key: Vec<u8> =
            index::make_ci_index_key(&event.created_at, event.id.as_bytes());
        let mut ci_index = txn.open_table(CI_INDEX)?;
        ci_index.insert(ci_index_key.as_slice(), id)?;

        // Index by author and kind (with created_at and id)
        let akc_index_key: Vec<u8> = index::make_akc_index_key(
            &event.pubkey.to_bytes(),
            event.kind.as_u16(),
            &event.created_at,
            event.id.as_bytes(),
        );
        let mut akc_index = txn.open_table(AKC_INDEX)?;
        akc_index.insert(akc_index_key.as_slice(), id)?;

        // Index by author (with created_at and id)
        let ac_index_key: Vec<u8> = index::make_ac_index_key(
            &event.pubkey.to_bytes(),
            &event.created_at,
            event.id.as_bytes(),
        );
        let mut ac_index = txn.open_table(AC_INDEX)?;
        ac_index.insert(ac_index_key.as_slice(), id)?;

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
                let mut atc_index = txn.open_table(ATC_INDEX)?;
                atc_index.insert(atc_index_key.as_slice(), id)?;

                // Index by kind and tag (with created_at and id)
                let ktc_index_key: Vec<u8> = index::make_ktc_index_key(
                    event.kind.as_u16(),
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id.as_bytes(),
                );
                let mut ktc_index = txn.open_table(KTC_INDEX)?;
                ktc_index.insert(ktc_index_key.as_slice(), id)?;

                // Index by tag (with created_at and id)
                let tc_index_key: Vec<u8> = index::make_tc_index_key(
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id.as_bytes(),
                );
                let mut tc_index = txn.open_table(TC_INDEX)?;
                tc_index.insert(tc_index_key.as_slice(), id)?;
            }
        }

        Ok(())
    }

    /// Remove the event
    pub(crate) fn remove(
        &self,
        txn: &WriteTransaction,
        event: &AccessGuardEvent,
    ) -> Result<(), Error> {
        let value = event.guard.value();
        let event = EventBorrow::decode(value)?;

        let mut events = txn.open_table(EVENTS)?;
        events.remove(event.id)?;

        let ci_index_key: Vec<u8> = index::make_ci_index_key(&event.created_at, event.id);
        let mut ci_index = txn.open_table(CI_INDEX)?;
        ci_index.remove(ci_index_key.as_slice())?;

        let akc_index_key: Vec<u8> =
            index::make_akc_index_key(event.pubkey, event.kind, &event.created_at, event.id);
        let mut akc_index = txn.open_table(AKC_INDEX)?;
        akc_index.remove(akc_index_key.as_slice())?;

        let ac_index_key: Vec<u8> =
            index::make_ac_index_key(event.pubkey, &event.created_at, event.id);
        let mut ac_index = txn.open_table(AC_INDEX)?;
        ac_index.remove(ac_index_key.as_slice())?;

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
                let mut atc_index = txn.open_table(ATC_INDEX)?;
                atc_index.remove(atc_index_key.as_slice())?;

                // Index by kind and tag (with created_at and id)
                let ktc_index_key: Vec<u8> = index::make_ktc_index_key(
                    event.kind,
                    &tag_name,
                    tag_value,
                    &event.created_at,
                    event.id,
                );
                let mut ktc_index = txn.open_table(KTC_INDEX)?;
                ktc_index.remove(ktc_index_key.as_slice())?;

                // Index by tag (with created_at and id)
                let tc_index_key: Vec<u8> =
                    index::make_tc_index_key(&tag_name, tag_value, &event.created_at, event.id);
                let mut tc_index = txn.open_table(TC_INDEX)?;
                tc_index.remove(tc_index_key.as_slice())?;
            }
        }

        Ok(())
    }

    pub(crate) fn wipe(&self, txn: &WriteTransaction) -> Result<(), Error> {
        // Delete tables
        txn.delete_table(EVENTS)?;
        txn.delete_table(CI_INDEX)?;
        txn.delete_table(TC_INDEX)?;
        txn.delete_table(AC_INDEX)?;
        txn.delete_table(AKC_INDEX)?;
        txn.delete_table(ATC_INDEX)?;
        txn.delete_table(KTC_INDEX)?;
        txn.delete_table(DELETED_IDS)?;
        txn.delete_table(DELETED_COORDINATES)?;

        // Re-create tables
        Self::create_tables(txn)?;

        Ok(())
    }

    #[inline]
    pub(crate) fn has_event(
        &self,
        txn: &ReadTransaction,
        event_id: &[u8; 32],
    ) -> Result<bool, Error> {
        Ok(self.get_event_by_id(txn, event_id)?.is_some())
    }

    #[inline]
    pub(crate) fn get_event_by_id<'a>(
        &self,
        txn: &ReadTransaction,
        event_id: &[u8; 32],
    ) -> Result<Option<AccessGuardEvent<'a>>, Error> {
        let events = txn.open_table(EVENTS)?;
        match events.get(event_id)? {
            Some(guard) => Ok(Some(AccessGuardEvent::new(guard)?)),
            None => Ok(None),
        }
    }

    pub fn query<'a, I>(
        &self,
        txn: &'a ReadTransaction,
        filters: I,
    ) -> Result<BTreeSet<AccessGuardEvent<'a>>, Error>
    where
        I: IntoIterator<Item = Filter>,
    {
        let mut output: BTreeSet<AccessGuardEvent<'a>> = BTreeSet::new();
        for filter in filters.into_iter() {
            let events = self.single_filter_query(txn, filter)?;
            output.extend(events);
        }
        Ok(output)
    }

    pub fn delete(
        &self,
        read_txn: &ReadTransaction,
        txn: &WriteTransaction,
        filter: Filter,
    ) -> Result<(), Error> {
        let events = self.single_filter_query(read_txn, filter)?;
        for event in events.into_iter() {
            self.remove(txn, &event)?;
        }
        Ok(())
    }

    /// Find all events that match the filter
    fn single_filter_query<'a>(
        &self,
        txn: &'a ReadTransaction,
        filter: Filter,
    ) -> Result<Box<dyn Iterator<Item = AccessGuardEvent<'a>> + 'a>, Error> {
        if let (Some(since), Some(until)) = (filter.since, filter.until) {
            if since > until {
                return Ok(Box::new(iter::empty()));
            }
        }

        // We insert into a BTreeSet to keep them time-ordered
        let mut output: BTreeSet<AccessGuardEvent<'a>> = BTreeSet::new();

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
                    if event.match_filter(&filter)? {
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
                        let id = value.value();
                        let event = self.get_event_by_id(txn, id)?.ok_or(Error::NotFound)?;

                        // If we have gone beyond since, we can stop early
                        // (We have to check because `since` might change in this loop)
                        if event.created_at < since {
                            break 'per_event;
                        }

                        // check against the rest of the filter
                        if event.match_filter(&filter)? {
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

                let (_key, guard) = result?;
                let id = guard.value();
                let event = self.get_event_by_id(txn, id)?.ok_or(Error::NotFound)?;

                if event.match_filter(&filter)? {
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
        txn: &ReadTransaction,
        filter: &DatabaseFilter,
        iter: IndexRange<'a>,
        since: &mut Timestamp,
        limit: Option<usize>,
        output: &mut BTreeSet<AccessGuardEvent<'a>>,
    ) -> Result<(), Error> {
        let mut count: usize = 0;

        for result in iter {
            let (_key, guard) = result?;

            // Get event by ID
            let id = guard.value();
            let event = self.get_event_by_id(txn, id)?.ok_or(Error::NotFound)?;

            if event.created_at < *since {
                break;
            }

            // check against the rest of the filter
            if event.match_filter(filter)? {
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
        txn: &ReadTransaction,
        author: &PublicKey,
        kind: Kind,
    ) -> Result<Option<AccessGuardEvent<'a>>, Error> {
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
            let (_key, guard) = result?;
            let id = guard.value();
            return self.get_event_by_id(txn, id);
        }

        Ok(None)
    }

    pub fn find_parameterized_replaceable_event<'a>(
        &'a self,
        txn: &ReadTransaction,
        addr: &Coordinate,
    ) -> Result<Option<AccessGuardEvent<'a>>, Error> {
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
            let (_key, guard) = result?;
            let id = guard.value();
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
        read_txn: &ReadTransaction,
        txn: &WriteTransaction,
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
            let (_key, guard) = result?;

            let id = guard.value();
            if let Some(event) = self.get_event_by_id(read_txn, id)? {
                self.remove(txn, &event)?;
            }
        }

        Ok(())
    }

    // Remove all parameterized-replaceable events with the matching author-kind-d
    // Kind must be a parameterized-replaceable event kind
    pub fn remove_parameterized_replaceable(
        &self,
        read_txn: &ReadTransaction,
        txn: &WriteTransaction,
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
            let (_key, guard) = result?;

            // Our index doesn't have Kind embedded, so we have to check it
            let id = guard.value();
            let event = self.get_event_by_id(read_txn, id)?.ok_or(Error::NotFound)?;

            if event.kind == coordinate.kind.as_u16() {
                self.remove(txn, &event)?;
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn is_deleted(
        &self,
        txn: &ReadTransaction,
        event_id: &EventId,
    ) -> Result<bool, Error> {
        let deleted_ids = txn.open_table(DELETED_IDS)?;
        Ok(deleted_ids.get(event_id.as_bytes())?.is_some())
    }

    pub(crate) fn mark_deleted(
        &self,
        txn: &WriteTransaction,
        event_id: &EventId,
    ) -> Result<(), Error> {
        let mut deleted_ids = txn.open_table(DELETED_IDS)?;
        deleted_ids.insert(event_id.as_bytes(), &())?;
        Ok(())
    }

    pub(crate) fn mark_coordinate_deleted(
        &self,
        txn: &WriteTransaction,
        coordinate: &CoordinateBorrow,
        when: Timestamp,
    ) -> Result<(), Error> {
        let key: Vec<u8> = index::make_coordinate_index_key(coordinate);
        let mut deleted_coordinates = txn.open_table(DELETED_COORDINATES)?;
        deleted_coordinates.insert(key.as_slice(), when.as_u64())?;
        Ok(())
    }

    pub(crate) fn when_is_coordinate_deleted(
        &self,
        txn: &ReadTransaction,
        coordinate: &CoordinateBorrow,
    ) -> Result<Option<Timestamp>, Error> {
        let key: Vec<u8> = index::make_coordinate_index_key(coordinate);
        self.when_is_coordinate_deleted_by_key(txn, key)
    }

    pub(crate) fn when_is_coordinate_deleted_by_key(
        &self,
        txn: &ReadTransaction,
        coordinate_key: Vec<u8>,
    ) -> Result<Option<Timestamp>, Error> {
        let deleted_coordinates = txn.open_table(DELETED_COORDINATES)?;
        Ok(deleted_coordinates
            .get(coordinate_key.as_slice())?
            .map(|guard| {
                let secs: u64 = guard.value();
                Timestamp::from_secs(secs)
            }))
    }

    pub(crate) fn ci_iter<'a>(
        &self,
        txn: &'a ReadTransaction,
        since: &Timestamp,
        until: &Timestamp,
    ) -> Result<IndexRange<'a>, Error> {
        let start_prefix = index::make_ci_index_key(until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_ci_index_key(since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        let ci_index = txn.open_table(CI_INDEX)?;
        Ok(ci_index.range::<&[u8]>(range)?)
    }

    pub(crate) fn tc_iter<'a>(
        &self,
        txn: &'a ReadTransaction,
        tag_name: &SingleLetterTag,
        tag_value: &str,
        since: &Timestamp,
        until: &Timestamp,
    ) -> Result<IndexRange<'a>, Error> {
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
        let tc_index = txn.open_table(TC_INDEX)?;
        Ok(tc_index.range::<&[u8]>(range)?)
    }

    pub(crate) fn ac_iter<'a>(
        &self,
        txn: &'a ReadTransaction,
        author: &[u8; 32],
        since: Timestamp,
        until: Timestamp,
    ) -> Result<IndexRange<'a>, Error> {
        let start_prefix = index::make_ac_index_key(author, &until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_ac_index_key(author, &since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        let ac_index = txn.open_table(AC_INDEX)?;
        Ok(ac_index.range::<&[u8]>(range)?)
    }

    pub(crate) fn akc_iter<'a>(
        &self,
        txn: &'a ReadTransaction,
        author: &[u8; 32],
        kind: u16,
        since: Timestamp,
        until: Timestamp,
    ) -> Result<IndexRange<'a>, Error> {
        let start_prefix = index::make_akc_index_key(author, kind, &until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_akc_index_key(author, kind, &since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(start_prefix.as_slice()),
            Bound::Excluded(end_prefix.as_slice()),
        );
        let akc_index = txn.open_table(AKC_INDEX)?;
        Ok(akc_index.range::<&[u8]>(range)?)
    }

    pub(crate) fn atc_iter<'a>(
        &self,
        txn: &'a ReadTransaction,
        author: &[u8; 32],
        tag_name: &SingleLetterTag,
        tag_value: &str,
        since: &Timestamp,
        until: &Timestamp,
    ) -> Result<IndexRange<'a>, Error> {
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
        let atc_index = txn.open_table(ATC_INDEX)?;
        Ok(atc_index.range::<&[u8]>(range)?)
    }

    pub(crate) fn ktc_iter<'a>(
        &self,
        txn: &'a ReadTransaction,
        kind: u16,
        tag_name: &SingleLetterTag,
        tag_value: &str,
        since: &Timestamp,
        until: &Timestamp,
    ) -> Result<IndexRange<'a>, Error> {
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
        let ktc_index = txn.open_table(KTC_INDEX)?;
        Ok(ktc_index.range::<&[u8]>(range)?)
    }
}
