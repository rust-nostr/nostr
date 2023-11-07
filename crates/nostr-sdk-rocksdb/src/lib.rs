// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use nostr::event::raw::RawEvent;
use nostr::{Event, EventId, Filter, FiltersMatchEvent, Timestamp, Url};
use nostr_sdk_db::{
    Backend, DatabaseError, DatabaseIndexes, DatabaseOptions, EventIndexResult, FlatBufferBuilder,
    FlatBufferDecode, FlatBufferEncode, NostrDatabase,
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
    indexes: DatabaseIndexes,
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
    pub fn new<P>(path: P) -> Result<Self, DatabaseError>
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

        Ok(Self {
            db: Arc::new(db),
            indexes: DatabaseIndexes::new(),
            fbb: Arc::new(RwLock::new(FlatBufferBuilder::with_capacity(70_000))),
        })
    }

    fn cf_handle(&self, name: &str) -> Result<Arc<BoundColumnFamily>, DatabaseError> {
        self.db.cf_handle(name).ok_or(DatabaseError::NotFound)
    }

    #[tracing::instrument(skip_all)]
    pub async fn build_indexes(&self) -> Result<(), DatabaseError> {
        let cf = self.cf_handle(EVENTS_CF)?;
        let events = self
            .db
            .full_iterator_cf(&cf, IteratorMode::Start)
            .flatten()
            .filter_map(|(_, value)| RawEvent::decode(&value).ok());
        self.indexes.bulk_load(events).await;
        Ok(())
    }
}

#[async_trait]
impl NostrDatabase for RocksDatabase {
    type Err = DatabaseError;

    fn backend(&self) -> Backend {
        Backend::RocksDB
    }

    fn opts(&self) -> DatabaseOptions {
        DatabaseOptions::default()
    }

    async fn count(&self) -> Result<usize, Self::Err> {
        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            let cf = this.cf_handle(EVENTS_CF)?;
            Ok(this
                .db
                .full_iterator_cf(&cf, IteratorMode::Start)
                .flatten()
                .count())
        })
        .await
        .unwrap()
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        // Index event
        let EventIndexResult {
            to_store,
            to_discard,
        } = self.indexes.index_event(event).await;

        if to_store {
            // Acquire FlatBuffers Builder
            let mut fbb = self.fbb.write().await;

            tokio::task::block_in_place(|| {
                // Get Column Families
                let events_cf = self.cf_handle(EVENTS_CF)?;

                // Serialize key and value
                let key: &[u8] = event.id.as_bytes();
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

    async fn has_event_already_been_seen(&self, event_id: EventId) -> Result<bool, Self::Err> {
        let cf = self.cf_handle(EVENTS_CF)?;
        Ok(self.db.key_may_exist_cf(&cf, event_id.as_bytes()))
    }

    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: Option<Url>,
    ) -> Result<(), Self::Err> {
        let mut fbb = self.fbb.write().await;
        let cf = self.cf_handle(EVENTS_SEEN_BY_RELAYS_CF)?;
        let value: HashSet<Url> = match relay_url {
            Some(relay_url) => {
                let mut set = HashSet::with_capacity(1);
                set.insert(relay_url);
                set
            }
            None => HashSet::new(),
        };
        self.db
            .merge_cf(&cf, event_id, value.encode(&mut fbb))
            .map_err(DatabaseError::backend)
    }

    async fn event_recently_seen_on_relays(
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

    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip_all)]
    async fn query(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Self::Err> {
        let ids = self.indexes.query(filters.clone()).await;

        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            let cf = this.cf_handle(EVENTS_CF)?;

            let mut events: Vec<Event> = Vec::new();

            for v in this
                .db
                .batched_multi_get_cf(&cf, ids, false)
                .into_iter()
                .flatten()
                .flatten()
            {
                let event: Event = Event::decode(&v).map_err(DatabaseError::backend)?;
                if filters.match_event(&event) {
                    events.push(event);
                }
            }

            Ok(events)
        })
        .await
        .map_err(DatabaseError::backend)?
    }

    async fn event_ids_by_filters(
        &self,
        filters: Vec<Filter>,
    ) -> Result<HashSet<EventId>, Self::Err> {
        Ok(self.indexes.query(filters).await)
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
        let ids = self.indexes.query(vec![filter.clone()]).await;

        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            let cf = this.cf_handle(EVENTS_CF)?;

            let mut event_ids: Vec<(EventId, Timestamp)> = Vec::new();

            for v in this
                .db
                .batched_multi_get_cf(&cf, ids, false)
                .into_iter()
                .flatten()
                .flatten()
            {
                let event: Event = Event::decode(&v).map_err(DatabaseError::backend)?;
                if filter.match_event(&event) {
                    event_ids.push((event.id, event.created_at));
                }
            }

            Ok(event_ids)
        })
        .await
        .map_err(DatabaseError::backend)?
    }

    async fn wipe(&self) -> Result<(), Self::Err> {
        Err(DatabaseError::NotSupported)
    }
}
