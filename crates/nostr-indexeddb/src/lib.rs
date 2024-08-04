// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Web's IndexedDB Storage backend for Nostr SDK

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints, clippy::arc_with_non_send_sync)]
#![cfg_attr(not(target_arch = "wasm32"), allow(unused))]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::future::IntoFuture;
use std::sync::Arc;

pub extern crate nostr;
pub extern crate nostr_database as database;

#[cfg(target_arch = "wasm32")]
use async_trait::async_trait;
use indexed_db_futures::js_sys::JsString;
use indexed_db_futures::request::{IdbOpenDbRequestLike, OpenDbRequest};
use indexed_db_futures::web_sys::IdbTransactionMode;
use indexed_db_futures::{IdbDatabase, IdbQuerySource, IdbVersionChangeEvent};
use nostr::nips::nip01::Coordinate;
use nostr::util::hex;
use nostr::{Event, EventId, Filter, Timestamp, Url};
#[cfg(target_arch = "wasm32")]
use nostr_database::NostrDatabase;
use nostr_database::{
    Backend, DatabaseError, DatabaseIndexes, EventIndexResult, FlatBufferBuilder, FlatBufferDecode,
    FlatBufferEncode, Order,
};
use tokio::sync::Mutex;
use wasm_bindgen::{JsCast, JsValue};

mod error;

pub use self::error::IndexedDBError;

const CURRENT_DB_VERSION: u32 = 2;
const EVENTS_CF: &str = "events";
const EVENTS_SEEN_BY_RELAYS_CF: &str = "event-seen-by-relays";
const ALL_STORES: [&str; 2] = [EVENTS_CF, EVENTS_SEEN_BY_RELAYS_CF];

/// Helper struct for upgrading the inner DB.
#[derive(Debug, Clone, Default)]
pub struct OngoingMigration {
    /// Names of stores to drop.
    drop_stores: HashSet<&'static str>,
    /// Names of stores to create.
    create_stores: HashSet<&'static str>,
    /// Store name => key-value data to add.
    data: HashMap<&'static str, Vec<(JsValue, JsValue)>>,
}

/// IndexedDB Nostr Database
#[derive(Clone)]
pub struct WebDatabase {
    db: Arc<IdbDatabase>,
    indexes: DatabaseIndexes,
    fbb: Arc<Mutex<FlatBufferBuilder<'static>>>,
}

impl fmt::Debug for WebDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebDatabase")
            .field("name", &self.db.name())
            .finish()
    }
}

impl WebDatabase {
    /// Open IndexedDB store
    pub async fn open<S>(name: S) -> Result<Self, IndexedDBError>
    where
        S: AsRef<str>,
    {
        let mut this = Self {
            db: Arc::new(IdbDatabase::open(name.as_ref())?.into_future().await?),
            indexes: DatabaseIndexes::new(),
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::with_capacity(70_000))),
        };

        this.migration().await?;
        this.build_indexes().await?;

        Ok(this)
    }

    async fn migration(&mut self) -> Result<(), IndexedDBError> {
        let name: String = self.db.name();
        let mut old_version: u32 = self.db.version() as u32;

        if old_version < CURRENT_DB_VERSION {
            // Inside the `onupgradeneeded` callback we would know whether it's a new DB
            // because the old version would be set to 0, here it is already set to 1 so we
            // check if the stores exist.
            if old_version == 1 && self.db.object_store_names().next().is_none() {
                old_version = 0;
            }

            if old_version == 0 {
                tracing::info!("Initializing database schemas...");
                let migration = OngoingMigration {
                    create_stores: ALL_STORES.into_iter().collect(),
                    ..Default::default()
                };
                self.apply_migration(CURRENT_DB_VERSION, migration).await?;
                tracing::info!("Database schemas initialized.");
            } else {
                /* if old_version < 3 {
                    self.migrate_to_v3().await?;
                }

                if old_version < 4 {} */
            }

            self.db.close();

            let mut db_req: OpenDbRequest = IdbDatabase::open_u32(&name, CURRENT_DB_VERSION)?;
            db_req.set_on_upgrade_needed(Some(
                move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
                    // Sanity check.
                    // There should be no upgrade needed since the database should have already been
                    // upgraded to the latest version.
                    panic!(
                        "Opening database that was not fully upgraded: \
                         DB version: {}; latest version: {CURRENT_DB_VERSION}",
                        evt.old_version()
                    )
                },
            ));

            self.db = Arc::new(db_req.into_future().await?);
        }

        Ok(())
    }

    async fn apply_migration(
        &mut self,
        version: u32,
        migration: OngoingMigration,
    ) -> Result<(), IndexedDBError> {
        let name: String = self.db.name();
        self.db.close();

        let mut db_req: OpenDbRequest = IdbDatabase::open_u32(&name, version)?;
        db_req.set_on_upgrade_needed(Some(
            move |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
                // Changing the format can only happen in the upgrade procedure
                for store in &migration.drop_stores {
                    evt.db().delete_object_store(store)?;
                }
                for store in &migration.create_stores {
                    evt.db().create_object_store(store)?;
                    tracing::debug!("Created '{store}' object store");
                }

                Ok(())
            },
        ));

        self.db = Arc::new(db_req.into_future().await?);

        // Finally, we can add data to the newly created tables if needed.
        if !migration.data.is_empty() {
            let stores: Vec<&str> = migration.data.keys().copied().collect();
            let tx = self
                .db
                .transaction_on_multi_with_mode(&stores, IdbTransactionMode::Readwrite)?;

            for (name, data) in migration.data {
                let store = tx.object_store(name)?;
                for (key, value) in data {
                    store.put_key_val(&key, &value)?;
                }
            }

            tx.await.into_result()?;
        }

        Ok(())
    }

    async fn build_indexes(&self) -> Result<(), IndexedDBError> {
        tracing::debug!("Building database indexes...");
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(EVENTS_CF)?;
        let events = store
            .get_all()?
            .await?
            .into_iter()
            .filter_map(js_value_to_string)
            .filter_map(|v| {
                let bytes = hex::decode(v).ok()?;
                Event::decode(&bytes).ok()
            });

        // Build indexes
        let to_discard: HashSet<EventId> = self.indexes.bulk_index(events.collect()).await;

        // Discard events
        for event_id in to_discard.into_iter() {
            let key = JsValue::from(event_id.to_hex());
            store.delete(&key)?.await?;
        }

        tracing::info!("Database indexes loaded");
        Ok(())
    }
}

// Small hack to have the following macro invocation act as the appropriate
// trait impl block on wasm, but still be compiled on non-wasm as a regular
// impl block otherwise.
//
// The trait impl doesn't compile on non-wasm due to unfulfilled trait bounds,
// this hack allows us to still have most of rust-analyzer's IDE functionality
// within the impl block without having to set it up to check things against
// the wasm target (which would disable many other parts of the codebase).
#[cfg(target_arch = "wasm32")]
macro_rules! impl_nostr_database {
    ({ $($body:tt)* }) => {
        #[async_trait(?Send)]
        impl NostrDatabase for WebDatabase {
            type Err = IndexedDBError;

            $($body)*
        }
    };
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! impl_nostr_database {
    ({ $($body:tt)* }) => {
        impl WebDatabase {
            $($body)*
        }
    };
}

impl_nostr_database!({
    fn backend(&self) -> Backend {
        Backend::IndexedDB
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn save_event(&self, event: &Event) -> Result<bool, IndexedDBError> {
        // Index event
        let EventIndexResult {
            to_store,
            to_discard,
        } = self.indexes.index_event(event).await;

        if to_store {
            let tx = self
                .db
                .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readwrite)?;
            let store = tx.object_store(EVENTS_CF)?;
            let key = JsValue::from(event.id().to_hex());

            // Acquire FlatBuffers Builder
            let mut fbb = self.fbb.lock().await;

            // Encode
            let event_hex: String = hex::encode(event.encode(&mut fbb));

            // Drop FlatBuffers Builder
            drop(fbb);

            // Store key-val
            let value = JsValue::from(event_hex);
            store.put_key_val(&key, &value)?;

            // Discard events no longer needed
            for event_id in to_discard.into_iter() {
                let key = JsValue::from(event_id.to_hex());
                store.delete(&key)?;
            }

            tx.await.into_result()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn bulk_import(&self, events: BTreeSet<Event>) -> Result<(), IndexedDBError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(EVENTS_CF)?;

        // Bulk import indexes
        let events = self.indexes.bulk_import(events).await;

        // Acquire FlatBuffers Builder
        let mut fbb = self.fbb.lock().await;

        for event in events.into_iter() {
            let key = JsValue::from(event.id.to_hex());
            let value = JsValue::from(hex::encode(event.encode(&mut fbb)));
            store.put_key_val(&key, &value)?.await?;
        }

        // Drop FlatBuffers Builder
        drop(fbb);

        tx.await.into_result()?;

        Ok(())
    }

    async fn has_event_already_been_saved(
        &self,
        event_id: &EventId,
    ) -> Result<bool, IndexedDBError> {
        if self.indexes.has_event_id_been_deleted(event_id).await {
            Ok(true)
        } else {
            let tx = self
                .db
                .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readonly)?;
            let store = tx.object_store(EVENTS_CF)?;
            let key = JsValue::from(event_id.to_hex());
            Ok(store.get(&key)?.await?.is_some())
        }
    }

    async fn has_event_already_been_seen(
        &self,
        event_id: &EventId,
    ) -> Result<bool, IndexedDBError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_SEEN_BY_RELAYS_CF, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(EVENTS_SEEN_BY_RELAYS_CF)?;
        let key = JsValue::from(event_id.to_hex());
        Ok(store.get(&key)?.await?.is_some())
    }

    async fn has_event_id_been_deleted(&self, event_id: &EventId) -> Result<bool, IndexedDBError> {
        Ok(self.indexes.has_event_id_been_deleted(event_id).await)
    }

    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: Timestamp,
    ) -> Result<bool, IndexedDBError> {
        Ok(self
            .indexes
            .has_coordinate_been_deleted(coordinate, timestamp)
            .await)
    }

    async fn event_id_seen(&self, event_id: EventId, relay_url: Url) -> Result<(), IndexedDBError> {
        let mut set: HashSet<Url> = self
            .event_seen_on_relays(event_id)
            .await?
            .unwrap_or_else(|| HashSet::with_capacity(1));

        if set.insert(relay_url) {
            let tx = self.db.transaction_on_one_with_mode(
                EVENTS_SEEN_BY_RELAYS_CF,
                IdbTransactionMode::Readwrite,
            )?;
            let store = tx.object_store(EVENTS_SEEN_BY_RELAYS_CF)?;
            let key = JsValue::from(event_id.to_hex());

            // Acquire FlatBuffers Builder
            let mut fbb = self.fbb.lock().await;

            // Encode
            let value = JsValue::from(hex::encode(set.encode(&mut fbb)));

            // Drop FlatBuffers Builder
            drop(fbb);

            // Save
            store.put_key_val(&key, &value)?.await?;
        }

        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        event_id: EventId,
    ) -> Result<Option<HashSet<Url>>, IndexedDBError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_SEEN_BY_RELAYS_CF, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(EVENTS_SEEN_BY_RELAYS_CF)?;
        let key = JsValue::from(event_id.to_hex());
        match store.get(&key)?.await? {
            Some(jsvalue) => {
                let set_hex = js_value_to_string(jsvalue)
                    .ok_or(IndexedDBError::Database(DatabaseError::NotFound))?;
                let bytes = hex::decode(set_hex).map_err(DatabaseError::backend)?;
                Ok(Some(
                    HashSet::decode(&bytes).map_err(DatabaseError::backend)?,
                ))
            }
            None => Ok(None),
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn event_by_id(&self, event_id: EventId) -> Result<Event, IndexedDBError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(EVENTS_CF)?;
        let key = JsValue::from(event_id.to_hex());
        match store.get(&key)?.await? {
            Some(jsvalue) => {
                let event_hex: String = js_value_to_string(jsvalue)
                    .ok_or(IndexedDBError::Database(DatabaseError::NotFound))?;
                let bytes: Vec<u8> = hex::decode(event_hex).map_err(DatabaseError::backend)?;
                Ok(Event::decode(&bytes).map_err(DatabaseError::backend)?)
            }
            None => Err(IndexedDBError::Database(DatabaseError::NotFound)),
        }
    }

    async fn count(&self, filters: Vec<Filter>) -> Result<usize, IndexedDBError> {
        Ok(self.indexes.count(filters).await)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    async fn query(
        &self,
        filters: Vec<Filter>,
        order: Order,
    ) -> Result<Vec<Event>, IndexedDBError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(EVENTS_CF)?;

        let ids: Vec<EventId> = self.indexes.query(filters, order).await;
        let mut events: Vec<Event> = Vec::with_capacity(ids.len());

        for event_id in ids.into_iter() {
            let key = JsValue::from(event_id.to_hex());
            if let Some(jsvalue) = store.get(&key)?.await? {
                let event_hex: String =
                    js_value_to_string(jsvalue).ok_or(DatabaseError::NotFound)?;
                let bytes: Vec<u8> = hex::decode(event_hex).map_err(DatabaseError::backend)?;
                let event: Event = Event::decode(&bytes).map_err(DatabaseError::backend)?;
                events.push(event);
            }
        }

        Ok(events)
    }

    async fn event_ids_by_filters(
        &self,
        filters: Vec<Filter>,
        order: Order,
    ) -> Result<Vec<EventId>, IndexedDBError> {
        Ok(self.indexes.query(filters, order).await)
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, IndexedDBError> {
        Ok(self.indexes.negentropy_items(filter).await)
    }

    async fn delete(&self, filter: Filter) -> Result<(), IndexedDBError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(EVENTS_CF)?;

        match self.indexes.delete(filter).await {
            Some(ids) => {
                for id in ids.into_iter() {
                    let key = JsValue::from(id.to_hex());
                    store.delete(&key)?.await?;
                }
            }
            None => {
                store.clear()?.await?;
            }
        };

        Ok(())
    }

    async fn wipe(&self) -> Result<(), IndexedDBError> {
        for store in ALL_STORES.iter() {
            let tx = self
                .db
                .transaction_on_one_with_mode(store, IdbTransactionMode::Readwrite)?;
            let store = tx.object_store(store)?;
            store.clear()?.await?;
        }

        self.indexes.clear().await;

        Ok(())
    }
});

#[inline(always)]
fn js_value_to_string(value: JsValue) -> Option<String> {
    let s: JsString = value.dyn_into().ok()?;
    Some(s.into())
}
