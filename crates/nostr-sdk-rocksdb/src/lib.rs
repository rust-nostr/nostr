// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use nostr::FiltersMatchEvent;
use nostr_sdk_db::nostr::{Event, EventId, Filter, Timestamp, Url};
use nostr_sdk_db::{async_trait, Backend, DatabaseError, DatabaseOptions, NostrDatabase};
use nostr_sdk_fbs::{FlatBufferBuilder, FlatBufferUtils};
use rocksdb::{
    BoundColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType,
    OptimisticTransactionDB, Options, WriteBatchWithTransaction,
};
use tokio::sync::RwLock;

mod ops;

use self::ops::indexes_merge_operator;

const EVENTS_CF: &str = "events";
const PUBKEY_INDEX_CF: &str = "pubkey_index";
const KIND_INDEX_CF: &str = "kind_index";

/// RocksDB Nostr Database
#[derive(Debug, Clone)]
pub struct RocksDatabase {
    db: Arc<OptimisticTransactionDB>,
    fbb: Arc<RwLock<FlatBufferBuilder<'static>>>,
}

fn default_opts() -> rocksdb::Options {
    let mut opts = Options::default();
    opts.set_keep_log_file_num(10);
    opts.set_max_open_files(100);
    opts.set_compaction_style(DBCompactionStyle::Level);
    opts.set_compression_type(DBCompressionType::Snappy);
    opts.set_target_file_size_base(256 << 20);
    opts.set_write_buffer_size(256 << 20);
    opts.set_enable_write_thread_adaptive_yield(true);
    opts.set_disable_auto_compactions(false);
    opts.increase_parallelism(2);
    opts
}

fn column_families() -> Vec<ColumnFamilyDescriptor> {
    let mut index_opts: Options = default_opts();
    index_opts.set_merge_operator_associative("index_merge_operator", indexes_merge_operator);

    vec![
        ColumnFamilyDescriptor::new(EVENTS_CF, default_opts()),
        ColumnFamilyDescriptor::new(PUBKEY_INDEX_CF, index_opts.clone()),
        ColumnFamilyDescriptor::new(KIND_INDEX_CF, index_opts),
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
            fbb: Arc::new(RwLock::new(FlatBufferBuilder::with_capacity(70_000))),
        })
    }

    fn cf_handle(&self, name: &str) -> Result<Arc<BoundColumnFamily>, DatabaseError> {
        self.db.cf_handle(name).ok_or(DatabaseError::NotFound)
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

    #[tracing::instrument(skip_all)]
    async fn save_event(&self, event: &Event) -> Result<bool, Self::Err> {
        // Acquire FlatBuffers Builder
        let mut fbb = self.fbb.write().await;

        // Get Column Families
        let events_cf = self.cf_handle(EVENTS_CF)?;
        let pubkey_index_cf = self.cf_handle(PUBKEY_INDEX_CF)?;
        let kind_index_cf = self.cf_handle(KIND_INDEX_CF)?;

        // Serialize key and value
        let key: &[u8] = event.id.as_bytes();
        let value: &[u8] = event.encode(&mut fbb);

        // Prepare write batch
        let mut batch = WriteBatchWithTransaction::default();

        // Save event
        batch.put_cf(&events_cf, key, value);

        // Save pubkey index
        batch.merge_cf(&pubkey_index_cf, event.pubkey.serialize(), key);

        // Save kind index
        batch.merge_cf(&kind_index_cf, event.kind.as_u64().to_be_bytes(), key);

        // Write batch changes
        self.db.write(batch).map_err(DatabaseError::backend)?;

        // Return status
        Ok(true)
    }

    async fn has_event_already_been_seen(&self, _event_id: EventId) -> Result<bool, Self::Err> {
        todo!()
    }

    async fn event_id_seen(
        &self,
        _event_id: EventId,
        _relay_url: Option<Url>,
    ) -> Result<(), Self::Err> {
        todo!()
    }

    async fn event_ids_seen(
        &self,
        _event_ids: Vec<EventId>,
        _relay_url: Option<Url>,
    ) -> Result<(), Self::Err> {
        todo!()
    }

    async fn event_recently_seen_on_relays(
        &self,
        _event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, Self::Err> {
        todo!()
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
        let this = self.clone();
        tokio::task::spawn_blocking(move || {
            let mut events: Vec<Event> = Vec::new();

            let cf = this.cf_handle(EVENTS_CF)?;
            let kind_index_cf = this.cf_handle(KIND_INDEX_CF)?;

            let mut ids_to_get: HashSet<[u8; 32]> = HashSet::new();

            let filter = filters.first().unwrap();
            if !filter.kinds.is_empty() {
                let keys = filter.kinds.iter().map(|k| k.as_u64().to_be_bytes());
                for v in this
                    .db
                    .batched_multi_get_cf(&kind_index_cf, keys, false)
                    .into_iter()
                    .flatten()
                    .flatten()
                {
                    let set: HashSet<[u8; 32]> =
                        HashSet::decode(&v).map_err(DatabaseError::backend)?;
                    ids_to_get.extend(set);
                }
            } else {
                tracing::debug!("No kinds set to query");
            }

            for v in this
                .db
                .batched_multi_get_cf(&cf, ids_to_get, false)
                .into_iter()
                .flatten()
                .flatten()
            {
                let event: Event = Event::decode(&v).map_err(DatabaseError::backend)?;
                events.push(event);
            }

            /* let iter = this.db.full_iterator_cf(&cf, IteratorMode::Start);

            for i in iter {
                if let Ok((_key, value)) = i {
                    let event: Event = Event::decode(&value).map_err(DatabaseError::backend)?;
                    if filters.match_event(&event) {
                        events.push(event);
                    }
                }
            } */

            /* iter.seek_to_first();
            while iter.valid() {
                if let Some(value) = iter.value() {
                    let event: Event = Event::decode(value).map_err(DatabaseError::backend)?;
                    if filters.match_event(&event) {
                        events.push(event);
                    }
                };
                iter.next();
            } */

            Ok(events)
        })
        .await
        .map_err(DatabaseError::backend)?
    }

    async fn event_ids_by_filters(&self, _filters: Vec<Filter>) -> Result<Vec<EventId>, Self::Err> {
        todo!()
    }

    async fn negentropy_items(
        &self,
        _filter: &Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, Self::Err> {
        todo!()
    }

    async fn wipe(&self) -> Result<(), Self::Err> {
        todo!()
    }
}
