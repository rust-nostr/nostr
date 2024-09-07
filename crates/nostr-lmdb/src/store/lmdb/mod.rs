// Copyright (c) 2024 Michael Dilger
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

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
use super::types::DatabaseEvent;

const EVENT_ID_ALL_ZEROS: [u8; 32] = [0; 32];
const EVENT_ID_ALL_255: [u8; 32] = [255; 32];

#[derive(Debug)]
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
                .map_size(1048576 * 1024 * 24) // 24 GB
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

    // /// Sync the data to disk. This happens periodically, but sometimes it's useful to force
    // /// it.
    // pub(crate) fn sync(&self) -> Result<(), Error> {
    //     self.env.force_sync()?;
    //     Ok(())
    // }

    // pub(crate) fn close(self) -> Result<(), Error> {
    //     self.env.force_sync()?;
    //     let closing_event = self.env.prepare_for_closing();
    //     closing_event.wait();
    //     Ok(())
    // }

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
            if let Some(tag_name) = tag.single_letter_tag() {
                if let Some(tag_value) = tag.content() {
                    // Index by tag (with created_at and id)
                    let tc_index_key: Vec<u8> = index::make_tc_index_key(
                        &tag_name,
                        tag_value,
                        &event.created_at,
                        event.id.as_bytes(),
                    );
                    self.tc_index.put(txn, &tc_index_key, id)?;

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
                }
            }
        }

        Ok(())
    }

    /// Remove the event
    pub(crate) fn remove(&self, txn: &mut RwTxn, event: &DatabaseEvent) -> Result<(), Error> {
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

        let ac_index_key: Vec<u8> =
            index::make_ac_index_key(event.author(), &event.created_at, event.id());
        self.ac_index.delete(txn, &ac_index_key)?;

        let ci_index_key: Vec<u8> = index::make_ci_index_key(&event.created_at, event.id());
        self.ci_index.delete(txn, &ci_index_key)?;

        let akc_index_key: Vec<u8> =
            index::make_akc_index_key(event.author(), event.kind, &event.created_at, event.id());
        self.akc_index.delete(txn, &akc_index_key)?;

        self.events.delete(txn, event.id())?;

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
    ) -> Result<RoRange<'_, Bytes, Bytes>, Error> {
        let start_prefix = index::make_ci_index_key(until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_ci_index_key(since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(&*start_prefix),
            Bound::Excluded(&*end_prefix),
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
    ) -> Result<RoRange<'_, Bytes, Bytes>, Error> {
        let start_prefix = index::make_tc_index_key(
            tag_name,
            tag_value,
            until, // scan goes backwards in time
            &EVENT_ID_ALL_ZEROS,
        );
        let end_prefix = index::make_tc_index_key(tag_name, tag_value, since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(&*start_prefix),
            Bound::Excluded(&*end_prefix),
        );
        Ok(self.tc_index.range(txn, &range)?)
    }

    pub(crate) fn ac_iter<'a>(
        &'a self,
        txn: &'a RoTxn,
        author: &[u8; 32],
        since: Timestamp,
        until: Timestamp,
    ) -> Result<RoRange<'_, Bytes, Bytes>, Error> {
        let start_prefix = index::make_ac_index_key(author, &until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_ac_index_key(author, &since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(&*start_prefix),
            Bound::Excluded(&*end_prefix),
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
    ) -> Result<RoRange<'_, Bytes, Bytes>, Error> {
        let start_prefix = index::make_akc_index_key(author, kind, &until, &EVENT_ID_ALL_ZEROS);
        let end_prefix = index::make_akc_index_key(author, kind, &since, &EVENT_ID_ALL_255);
        let range = (
            Bound::Included(&*start_prefix),
            Bound::Excluded(&*end_prefix),
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
    ) -> Result<RoRange<'_, Bytes, Bytes>, Error> {
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
            Bound::Included(&*start_prefix),
            Bound::Excluded(&*end_prefix),
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
    ) -> Result<RoRange<'_, Bytes, Bytes>, Error> {
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
            Bound::Included(&*start_prefix),
            Bound::Excluded(&*end_prefix),
        );
        Ok(self.ktc_index.range(txn, &range)?)
    }
}
