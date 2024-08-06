// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! RocksDB Storage backend for Nostr SDK

#![forbid(unsafe_code)]
#![deny(warnings)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use std::sync::Arc;

pub extern crate nostr;
pub extern crate nostr_database as database;

use async_trait::async_trait;
use nostr::nips::nip01::Coordinate;
use nostr::{Event, EventId, Filter, Timestamp, Url};
use nostr_database::{
    Backend, DatabaseError, DatabaseEventResult, DatabaseHelper, FlatBufferBuilder,
    FlatBufferDecode, FlatBufferEncode, NostrDatabase, Order,
};
use rocksdb::{
    BoundColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType, IteratorMode,
    OptimisticTransactionDB, Options, WriteBatchWithTransaction,
};
use tokio::sync::RwLock;

mod ops;

const EVENTS_CF: &str = "events";
const EVENTS_SEEN_BY_RELAYS_CF: &str = "event-seen-by-relays";

/// RocksDB Nostr Database
#[derive(Debug, Clone)]
pub struct RocksDatabase {
    db: Arc<OptimisticTransactionDB>,
    helper: DatabaseHelper,
    fbb: Arc<RwLock<FlatBufferBuilder<'static>>>,
}

fn default_opts() -> rocksdb::Options {
    let mut opts = Options::default();
    opts.set_keep_log_file_num(10);
    opts.set_max_open_files(16);
    opts.set_compaction_style(DBCompactionStyle::Level);
    opts.set_compression_type(DBCompressionType::Snappy);
    opts.set_target_file_size_base(64 * 1024 * 1024); // 64 MB
    opts.set_write_buffer_size(64 * 1024 * 1024); // 64 MB
    opts.set_enable_write_thread_adaptive_yield(true);
    opts.set_disable_auto_compactions(false);
    opts.increase_parallelism(num_cpus::get() as i32);
    opts
}

fn column_families() -> Vec<ColumnFamilyDescriptor> {
    let mut relay_urls_opts: Options = default_opts();
    relay_urls_opts.set_merge_operator_associative(
        "relay_urls_merge_operator",
        ops::relay_urls_merge_operator,
    );

    vec![
        ColumnFamilyDescriptor::new(EVENTS_CF, default_opts()),
        ColumnFamilyDescriptor::new(EVENTS_SEEN_BY_RELAYS_CF, relay_urls_opts),
    ]
}

impl RocksDatabase {
    /// Open RocksDB store
    pub async fn open<P>(path: P) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        let path: &Path = path.as_ref();

        tracing::debug!("Opening {}", path.display());

        let mut db_opts = default_opts();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let db = OptimisticTransactionDB::open_cf_descriptors(&db_opts, path, column_families())
            .map_err(DatabaseError::backend)?;

        match db.live_files() {
            Ok(live_files) => tracing::info!(
                "{}: {} SST files, {} GB, {} Grows",
                path.display(),
                live_files.len(),
                live_files.iter().map(|f| f.size).sum::<usize>() as f64 / 1e9,
                live_files.iter().map(|f| f.num_entries).sum::<u64>() as f64 / 1e9
            ),
            Err(_) => tracing::warn!("Impossible to get live files"),
        };

        let this = Self {
            db: Arc::new(db),
            helper: DatabaseHelper::unbounded(),
            fbb: Arc::new(RwLock::new(FlatBufferBuilder::with_capacity(70_000))),
        };

        this.build_indexes().await?;

        Ok(this)
    }

    // TODO: add open_with_opts

    fn cf_handle(&self, name: &str) -> Result<Arc<BoundColumnFamily>, DatabaseError> {
        self.db.cf_handle(name).ok_or(DatabaseError::NotFound)
    }

    #[tracing::instrument(skip_all)]
    async fn build_indexes(&self) -> Result<(), DatabaseError> {
        let cf = self.cf_handle(EVENTS_CF)?;
        let events = self
            .db
            .full_iterator_cf(&cf, IteratorMode::Start)
            .flatten()
            .filter_map(|(_, value)| Event::decode(&value).ok())
            .collect();

        // Build indexes
        let to_discard: HashSet<EventId> = self.helper.bulk_index(events).await;

        // Discard events
        if !to_discard.is_empty() {
            // Prepare write batch
            let mut batch = WriteBatchWithTransaction::default();

            // Discard events no longer needed
            for event_id in to_discard.into_iter() {
                batch.delete_cf(&cf, event_id);
            }

            // Write batch changes
            self.db.write(batch).map_err(DatabaseError::backend)?;
        }

        Ok(())
    }
}

#[async_trait]
impl NostrDatabase for RocksDatabase {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        Backend::RocksDB
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        // Index event
        let DatabaseEventResult {
            to_store,
            to_discard,
        } = self.helper.index_event(event).await;

        if to_store {
            // Acquire FlatBuffers Builder
            let mut fbb = self.fbb.write().await;

            tokio::task::block_in_place(|| {
                // Get Column Families
                let events_cf = self.cf_handle(EVENTS_CF)?;

                // Serialize key and value
                let id = event.id();
                let key: &[u8] = id.as_bytes();
                let value: &[u8] = event.encode(&mut fbb);

                // Prepare write batch
                let mut batch = WriteBatchWithTransaction::default();

                // Save event
                batch.put_cf(&events_cf, key, value);

                // Discard events no longer needed
                for event_id in to_discard.into_iter() {
                    batch.delete_cf(&events_cf, event_id);
                }

                // Write batch changes
                self.db.write(batch).map_err(DatabaseError::backend)
            })?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn bulk_import(&self, events: BTreeSet<Event>) -> Result<(), Self::Err> {
        // Acquire FlatBuffers Builder
        let mut fbb = self.fbb.write().await;

        // Prepare write batch
        let mut batch = WriteBatchWithTransaction::default();

        let events = self.helper.bulk_import(events).await;

        // Get Column Family
        let events_cf = self.cf_handle(EVENTS_CF)?;

        for event in events.into_iter() {
            // Serialize key and value
            let id = event.id;
            let key: &[u8] = id.as_bytes();
            let value: &[u8] = event.encode(&mut fbb);

            // Save event
            batch.put_cf(&events_cf, key, value);
        }

        // Write batch changes
        self.db.write(batch).map_err(DatabaseError::backend)?;

        Ok(())
    }

    async fn has_event_already_been_saved(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        if self.helper.has_event_id_been_deleted(event_id).await {
            Ok(true)
        } else {
            let cf = self.cf_handle(EVENTS_CF)?;
            Ok(self.db.key_may_exist_cf(&cf, event_id.as_bytes()))
        }
    }

    async fn has_event_already_been_seen(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        let cf = self.cf_handle(EVENTS_SEEN_BY_RELAYS_CF)?;
        Ok(self.db.key_may_exist_cf(&cf, event_id.as_bytes()))
    }

    async fn has_event_id_been_deleted(&self, event_id: &EventId) -> Result<bool, Self::Err> {
        Ok(self.helper.has_event_id_been_deleted(event_id).await)
    }

    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: Timestamp,
    ) -> Result<bool, Self::Err> {
        Ok(self
            .helper
            .has_coordinate_been_deleted(coordinate, timestamp)
            .await)
    }

    async fn event_id_seen(&self, event_id: EventId, relay_url: Url) -> Result<(), Self::Err> {
        let mut fbb = self.fbb.write().await;
        let cf = self.cf_handle(EVENTS_SEEN_BY_RELAYS_CF)?;
        let value: HashSet<Url> = {
            let mut set = HashSet::with_capacity(1);
            set.insert(relay_url);
            set
        };
        self.db
            .merge_cf(&cf, event_id, value.encode(&mut fbb))
            .map_err(DatabaseError::backend)
    }

    async fn event_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err> {
        let cf = self.cf_handle(EVENTS_SEEN_BY_RELAYS_CF)?;
        match self
            .db
            .get_pinned_cf(&cf, event_id)
            .map_err(DatabaseError::backend)?
        {
            Some(val) => Ok(Some(HashSet::decode(&val).map_err(DatabaseError::backend)?)),
            None => Ok(None),
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn event_by_id(&self, event_id: EventId) -> Result<Event, Self::Err> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            let cf = this.cf_handle(EVENTS_CF)?;
            match this
                .db
                .get_pinned_cf(&cf, event_id.as_bytes())
                .map_err(DatabaseError::backend)?
            {
                Some(event) => Event::decode(&event).map_err(DatabaseError::backend),
                None => Err(DatabaseError::NotFound),
            }
        })
        .await
        .map_err(DatabaseError::backend)?
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, Self::Err> {
        Ok(self.helper.count(filters).await)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn query(&self, filters: Vec<Filter>, order: Order) -> Result<Vec<Event>, Self::Err> {
        Ok(self.helper.query(filters, order).await)
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
        Ok(self.helper.negentropy_items(filter).await)
    }

    async fn delete(&self, filter: Filter) -> Result<(), Self::Err> {
        match self.helper.delete(filter).await {
            Some(ids) => {
                let events_cf = self.cf_handle(EVENTS_CF)?;

                // Prepare write batch
                let mut batch = WriteBatchWithTransaction::default();

                for id in ids.into_iter() {
                    batch.delete_cf(&events_cf, id);
                }

                // Write batch changes
                self.db.write(batch).map_err(DatabaseError::backend)?;

                Ok(())
            }
            None => Err(DatabaseError::NotSupported),
        }
    }

    async fn wipe(&self) -> Result<(), Self::Err> {
        Err(DatabaseError::NotSupported)
    }
}
