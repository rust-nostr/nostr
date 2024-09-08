// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use heed::types::Bytes;
use heed::{RoRange, RoTxn, RwTxn};
use nostr::prelude::*;
use nostr_database::FlatBufferBuilder;
use tokio::sync::Mutex;

mod error;
mod lmdb;
mod types;

use self::error::Error;
use self::lmdb::Lmdb;
use crate::store::types::{DatabaseEvent, DatabaseFilter};

#[derive(Debug)]
pub struct Store {
    db: Lmdb,
    fbb: Arc<Mutex<FlatBufferBuilder<'static>>>,
}

impl Store {
    pub fn open<P>(path: P) -> Result<Store, Error>
    where
        P: AsRef<Path>,
    {
        let path: &Path = path.as_ref();

        // Create the directory if it doesn't exist
        fs::create_dir_all(path)?;

        Ok(Store {
            db: Lmdb::new(path)?,
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::with_capacity(70_000))),
        })
    }

    // /// Sync the data to disk. This happens periodically, but sometimes it's useful to force
    // /// it.
    // pub fn sync(&self) -> Result<(), Error> {
    //     self.db.sync()?;
    //     Ok(())
    // }

    /// Store an event.
    ///
    /// If the event already exists, you will get a Error::Duplicate
    ///
    /// If the event is ephemeral, it will be stored and you will get an offset, but
    /// it will not be indexed.
    pub async fn store_event(&self, event: &Event) -> Result<bool, Error> {
        if event.kind.is_ephemeral() {
            return Ok(false);
        }

        // Read operations
        {
            // Acquire read transaction
            let txn = self.db.read_txn()?;

            // Already exists
            if self.db.has_event(&txn, event.id.as_bytes())? {
                //return Err(Error::Duplicate);
                return Ok(false);
            }

            // Reject event if ID was deleted
            if self.db.is_deleted(&txn, &event.id)? {
                //return Err(Error::Deleted);
                return Ok(false);
            }

            // Reject event if ADDR was deleted after it's created_at date
            // (non-parameterized)
            if event.kind.is_replaceable() {
                let coordinate: Coordinate = Coordinate::new(event.kind, event.pubkey);
                if let Some(time) = self.db.when_is_coordinate_deleted(&txn, &coordinate)? {
                    if event.created_at <= time {
                        //return Err(Error::Deleted);
                        return Ok(false);
                    }
                }
            }

            // Reject event if ADDR was deleted after it's created_at date
            // (parameterized)
            if event.kind.is_parameterized_replaceable() {
                if let Some(identifier) = event.identifier() {
                    let coordinate: Coordinate =
                        Coordinate::new(event.kind, event.pubkey).identifier(identifier);
                    if let Some(time) = self.db.when_is_coordinate_deleted(&txn, &coordinate)? {
                        if event.created_at <= time {
                            //return Err(Error::Deleted);
                            return Ok(false);
                        }
                    }
                }
            }
        }

        // Acquire write transaction
        let mut txn = self.db.write_txn()?;

        // Pre-remove replaceable events being replaced
        {
            if event.kind.is_replaceable() {
                // Pre-remove any replaceable events that this replaces
                self.remove_replaceable(&mut txn, &event.pubkey, event.kind, event.created_at)?;

                // If any remaining matching replaceable events exist, then
                // this event is invalid, return Replaced
                if self
                    .find_replaceable_event(&txn, &event.pubkey, event.kind)?
                    .is_some()
                {
                    //return Err(Error::Replaced);
                    return Ok(false);
                }
            }

            if event.kind.is_parameterized_replaceable() {
                if let Some(identifier) = event.identifier() {
                    let coordinate: Coordinate =
                        Coordinate::new(event.kind, event.pubkey).identifier(identifier);

                    // Pre-remove any parameterized-replaceable events that this replaces
                    self.remove_parameterized_replaceable(&mut txn, &coordinate, Timestamp::max())?;

                    // If any remaining matching parameterized replaceable events exist, then
                    // this event is invalid, return Replaced
                    if self
                        .find_parameterized_replaceable_event(&txn, &coordinate)?
                        .is_some()
                    {
                        //return Err(Error::Replaced);
                        return Ok(false);
                    }
                }
            }
        }

        // Store
        {
            // Acquire flatbuffers builder
            let mut fbb = self.fbb.lock().await;

            // Store and index the event
            self.db.store(&mut txn, &mut fbb, event)?;

            // fbb dropped here
        }

        // Handle deletion events
        if let Kind::EventDeletion = event.kind {
            self.handle_deletion_event(&mut txn, event)?;
        }

        txn.commit()?;

        Ok(true)
    }

    fn handle_deletion_event(&self, txn: &mut RwTxn, event: &Event) -> Result<(), Error> {
        for id in event.event_ids() {
            // Actually remove
            if let Some(target) = self.db.get_event_by_id(txn, id.as_bytes())? {
                // author must match
                if target.author() != &event.pubkey.to_bytes() {
                    continue;
                }

                // Remove event
                self.remove_by_id(txn, id.as_bytes())?;
            }

            // Mark deleted
            // NOTE: if we didn't have the target event, we presume this is valid,
            //       and if not, clients will just have to deal with that.
            self.db.mark_deleted(txn, id)?;
        }

        for coordinate in event.coordinates() {
            if coordinate.public_key != event.pubkey {
                continue;
            }

            // Mark deleted
            self.db
                .mark_coordinate_deleted(txn, coordinate, event.created_at)?;

            // Remove events (up to the created_at of the deletion event)
            if coordinate.kind.is_replaceable() {
                self.remove_replaceable(
                    txn,
                    &coordinate.public_key,
                    coordinate.kind,
                    event.created_at,
                )?;
            } else if coordinate.kind.is_parameterized_replaceable() {
                self.remove_parameterized_replaceable(txn, coordinate, event.created_at)?;
            }
        }

        Ok(())
    }

    /// Get an event by ID
    pub fn get_event_by_id(&self, id: &EventId) -> Result<Option<Event>, Error> {
        let txn = self.db.read_txn()?;
        match self.db.get_event_by_id(&txn, id.as_bytes())? {
            Some(e) => Ok(Some(e.to_event()?)),
            None => Ok(None),
        }
    }

    /// Do we have an event
    pub fn has_event(&self, id: &EventId) -> Result<bool, Error> {
        let txn = self.db.read_txn()?;
        self.db.has_event(&txn, id.as_bytes())
    }

    /// Is the event deleted
    pub fn event_is_deleted(&self, id: &EventId) -> Result<bool, Error> {
        let txn = self.db.read_txn()?;
        self.db.is_deleted(&txn, id)
    }

    #[inline]
    pub fn when_is_coordinate_deleted(
        &self,
        coordinate: &Coordinate,
    ) -> Result<Option<Timestamp>, Error> {
        let txn = self.db.read_txn()?;
        self.db.when_is_coordinate_deleted(&txn, coordinate)
    }

    pub fn count<I>(&self, filters: I) -> Result<usize, Error>
    where
        I: IntoIterator<Item = Filter>,
    {
        let txn = self.db.read_txn()?;
        let output = self.gather_events(&txn, filters)?;
        Ok(output.len())
    }

    pub fn query<I>(&self, filters: I) -> Result<Vec<Event>, Error>
    where
        I: IntoIterator<Item = Filter>,
    {
        let txn = self.db.read_txn()?;
        let output = self.gather_events(&txn, filters)?;
        Ok(output
            .into_iter()
            .filter_map(|e| e.to_event().ok())
            .collect())
    }

    pub fn negentropy_items(&self, filter: Filter) -> Result<Vec<(EventId, Timestamp)>, Error> {
        let txn = self.db.read_txn()?;
        let events = self.single_filter_query(&txn, filter)?;
        Ok(events
            .into_iter()
            .map(|e| (EventId::from_byte_array(*e.id()), e.created_at))
            .collect())
    }

    fn gather_events<'a, I>(
        &self,
        txn: &'a RoTxn,
        filters: I,
    ) -> Result<BTreeSet<DatabaseEvent<'a>>, Error>
    where
        I: IntoIterator<Item = Filter>,
    {
        let mut output = BTreeSet::new();
        for filter in filters {
            let events = self.single_filter_query(txn, filter)?;
            output.extend(events);
        }
        Ok(output)
    }

    /// Find all events that match the filter
    fn single_filter_query<'a>(
        &self,
        txn: &'a RoTxn,
        filter: Filter,
    ) -> Result<Vec<DatabaseEvent<'a>>, Error> {
        if let (Some(since), Some(until)) = (filter.since, filter.until) {
            if since > until {
                return Ok(Vec::new());
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

                if let Some(event) = self.db.get_event_by_id(txn, &id.0)? {
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
                    let iter = self.db.akc_iter(txn, &author.0, *kind, since, until)?;

                    // Count how many we have found of this author-kind pair, so we
                    // can possibly update `since`
                    let mut paircount = 0;

                    'per_event: for result in iter {
                        let (_key, value) = result?;
                        let event = self
                            .db
                            .get_event_by_id(txn, value)?
                            .ok_or(Error::NotFound)?;

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
                        let iter = self
                            .db
                            .atc_iter(txn, &author.0, tagname, tag_value, &since, &until)?;
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
                        let iter = self
                            .db
                            .ktc_iter(txn, *kind, tag_name, tag_value, &since, &until)?;
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
                    let iter = self.db.tc_iter(txn, tag_name, tag_value, &since, &until)?;
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
                let iter = self.db.ac_iter(txn, &author.0, since, until)?;
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

            let iter = self.db.ci_iter(txn, &since, &until)?;
            for result in iter {
                // Check if limit is set
                if let Some(limit) = limit {
                    // Stop if limited
                    if output.len() >= limit {
                        break;
                    }
                }

                let (_key, value) = result?;
                let event = self
                    .db
                    .get_event_by_id(txn, value)?
                    .ok_or(Error::NotFound)?;

                if filter.match_event(&event) {
                    output.insert(event);
                }
            }
        }

        // Reverse order, optionally apply limit and collect to Vec
        Ok(match limit {
            Some(limit) => output.into_iter().take(limit).collect(),
            None => output.into_iter().collect(),
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
            let event = self
                .db
                .get_event_by_id(txn, value)?
                .ok_or(Error::NotFound)?;

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

    fn find_replaceable_event<'a>(
        &self,
        txn: &'a RoTxn,
        author: &PublicKey,
        kind: Kind,
    ) -> Result<Option<DatabaseEvent<'a>>, Error> {
        if !kind.is_replaceable() {
            return Err(Error::WrongEventKind);
        }

        let mut iter = self.db.akc_iter(
            txn,
            &author.to_bytes(),
            kind.as_u16(),
            Timestamp::min(),
            Timestamp::max(),
        )?;

        if let Some(result) = iter.next() {
            let (_key, value) = result?;
            return self.db.get_event_by_id(txn, value);
        }

        Ok(None)
    }

    fn find_parameterized_replaceable_event<'a>(
        &'a self,
        txn: &'a RoTxn,
        addr: &Coordinate,
    ) -> Result<Option<DatabaseEvent<'a>>, Error> {
        if !addr.kind.is_parameterized_replaceable() {
            return Err(Error::WrongEventKind);
        }

        let iter = self.db.atc_iter(
            txn,
            &addr.public_key.to_bytes(),
            &SingleLetterTag::lowercase(Alphabet::D),
            &addr.identifier,
            &Timestamp::min(),
            &Timestamp::max(),
        )?;

        for result in iter {
            let (_key, value) = result?;
            let event = self
                .db
                .get_event_by_id(txn, value)?
                .ok_or(Error::NotFound)?;

            // the atc index doesn't have kind, so we have to compare the kinds
            if event.kind != addr.kind.as_u16() {
                continue;
            }

            return Ok(Some(event));
        }

        Ok(None)
    }

    /// Remove an event by ID
    fn remove_by_id(&self, txn: &mut RwTxn, event_id: &[u8]) -> Result<(), Error> {
        let read_txn = self.db.read_txn()?;
        if let Some(event) = self.db.get_event_by_id(&read_txn, event_id)? {
            self.db.remove(txn, &event)?;
        }

        Ok(())
    }

    // Remove all replaceable events with the matching author-kind
    // Kind must be a replaceable (not parameterized replaceable) event kind
    fn remove_replaceable(
        &self,
        txn: &mut RwTxn,
        author: &PublicKey,
        kind: Kind,
        until: Timestamp,
    ) -> Result<(), Error> {
        if !kind.is_replaceable() {
            return Err(Error::WrongEventKind);
        }

        let read_txn = self.db.read_txn()?;
        let iter = self.db.akc_iter(
            &read_txn,
            &author.to_bytes(),
            kind.as_u16(),
            Timestamp::zero(),
            until,
        )?;

        for result in iter {
            let (_key, value) = result?;
            self.remove_by_id(txn, value)?;
        }

        Ok(())
    }

    // Remove all parameterized-replaceable events with the matching author-kind-d
    // Kind must be a paramterized-replaceable event kind
    fn remove_parameterized_replaceable(
        &self,
        txn: &mut RwTxn,
        coordinate: &Coordinate,
        until: Timestamp,
    ) -> Result<(), Error> {
        if !coordinate.kind.is_parameterized_replaceable() {
            return Err(Error::WrongEventKind);
        }

        let read_txn = self.db.read_txn()?;
        let iter = self.db.atc_iter(
            &read_txn,
            &coordinate.public_key.to_bytes(),
            &SingleLetterTag::lowercase(Alphabet::D),
            &coordinate.identifier,
            &Timestamp::min(),
            &until,
        )?;

        for result in iter {
            let (_key, value) = result?;

            // Our index doesn't have Kind embedded, so we have to check it
            let event = self
                .db
                .get_event_by_id(txn, value)?
                .ok_or(Error::NotFound)?;

            if event.kind == coordinate.kind.as_u16() {
                self.remove_by_id(txn, value)?;
            }
        }

        Ok(())
    }

    pub fn wipe(&self) -> Result<(), Error> {
        let mut txn = self.db.write_txn()?;
        self.db.wipe(&mut txn)?;
        txn.commit()?;
        Ok(())
    }
}
