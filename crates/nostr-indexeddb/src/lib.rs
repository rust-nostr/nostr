// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Web's IndexedDB Storage backend for Nostr SDK

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints, clippy::arc_with_non_send_sync)]
#![cfg_attr(not(target_arch = "wasm32"), allow(unused))]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::future::IntoFuture;
use std::sync::Arc;

pub extern crate nostr;
pub extern crate nostr_database as database;

use indexed_db_futures::js_sys::JsString;
use indexed_db_futures::request::{IdbOpenDbRequestLike, OpenDbRequest};
use indexed_db_futures::web_sys::IdbTransactionMode;
use indexed_db_futures::{IdbDatabase, IdbQuerySource, IdbVersionChangeEvent};
use nostr_database::prelude::*;
use tokio::sync::Mutex;
use wasm_bindgen::{JsCast, JsValue};

mod error;

use self::error::{into_err, IndexedDBError};

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
    helper: DatabaseHelper,
    fbb: Arc<Mutex<FlatBufferBuilder<'static>>>,
}

impl fmt::Debug for WebDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebDatabase")
            .field("name", &self.db.name())
            .finish()
    }
}

unsafe impl Send for WebDatabase {}

unsafe impl Sync for WebDatabase {}

impl WebDatabase {
    async fn new<S>(name: S, helper: DatabaseHelper) -> Result<Self, DatabaseError>
    where
        S: AsRef<str>,
    {
        let mut this = Self {
            db: Arc::new(
                IdbDatabase::open(name.as_ref())
                    .map_err(into_err)?
                    .into_future()
                    .await
                    .map_err(into_err)?,
            ),
            helper,
            fbb: Arc::new(Mutex::new(FlatBufferBuilder::with_capacity(70_000))),
        };

        this.migration().await?;
        this.bulk_load().await?;

        Ok(this)
    }

    /// Open database with **unlimited** capacity
    pub async fn open<S>(name: S) -> Result<Self, DatabaseError>
    where
        S: AsRef<str>,
    {
        Self::new(name, DatabaseHelper::unbounded()).await
    }

    /// Open database with **limited** capacity
    pub async fn open_bounded<S>(name: S, max_capacity: usize) -> Result<Self, DatabaseError>
    where
        S: AsRef<str>,
    {
        Self::new(name, DatabaseHelper::bounded(max_capacity)).await
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
                let migration = OngoingMigration {
                    create_stores: ALL_STORES.into_iter().collect(),
                    ..Default::default()
                };
                self.apply_migration(CURRENT_DB_VERSION, migration).await?;
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

    async fn bulk_load(&self) -> Result<(), IndexedDBError> {
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
            })
            .collect();

        // Build indexes
        let to_discard: HashSet<EventId> = self.helper.bulk_load(events).await;

        // Discard events
        for event_id in to_discard.into_iter() {
            let key = JsValue::from(event_id.to_hex());
            store.delete(&key)?.await?;
        }

        Ok(())
    }

    async fn _save_event(&self, event: &Event) -> Result<bool, IndexedDBError> {
        // Index event
        let DatabaseEventResult {
            to_store,
            to_discard,
        } = self.helper.index_event(event).await;

        if to_store {
            let tx = self
                .db
                .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readwrite)?;
            let store = tx.object_store(EVENTS_CF)?;
            let key = JsValue::from(event.id.to_hex());

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

    async fn _delete(&self, filter: Filter) -> Result<(), IndexedDBError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(EVENTS_CF)?;

        match self.helper.delete(filter).await {
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

    async fn _wipe(&self) -> Result<(), IndexedDBError> {
        for store in ALL_STORES.iter() {
            let tx = self
                .db
                .transaction_on_one_with_mode(store, IdbTransactionMode::Readwrite)?;
            let store = tx.object_store(store)?;
            store.clear()?.await?;
        }

        self.helper.clear().await;

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

// Small hack to have the following macro invocation act as the appropriate
// trait impl block on wasm, but still be compiled on non-wasm as a regular
// impl block otherwise.
//
// The trait impl doesn't compile on non-wasm due to unfulfilled trait bounds,
// this hack allows us to still have most of rust-analyzer's IDE functionality
// within the impl block without having to set it up to check things against
// the wasm target (which would disable many other parts of the codebase).
#[cfg(target_arch = "wasm32")]
macro_rules! impl_nostr_events_database {
    ({ $($body:tt)* }) => {
        #[async_trait(?Send)]
        impl NostrEventsDatabase for WebDatabase {
            $($body)*
        }
    };
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! impl_nostr_events_database {
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

    async fn wipe(&self) -> Result<(), DatabaseError> {
        self._wipe().await.map_err(DatabaseError::backend)
    }
});

impl_nostr_events_database!({
    async fn save_event(&self, event: &Event) -> Result<bool, DatabaseError> {
        self._save_event(event)
            .await
            .map_err(DatabaseError::backend)
    }

    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
        if self.helper.has_event_id_been_deleted(event_id).await {
            Ok(DatabaseEventStatus::Deleted)
        } else {
            let tx = self
                .db
                .transaction_on_one_with_mode(EVENTS_CF, IdbTransactionMode::Readonly)
                .map_err(into_err)?;
            let store = tx.object_store(EVENTS_CF).map_err(into_err)?;
            let key = JsValue::from(event_id.to_hex());
            Ok(
                if store
                    .get(&key)
                    .map_err(into_err)?
                    .await
                    .map_err(into_err)?
                    .is_some()
                {
                    DatabaseEventStatus::Saved
                } else {
                    DatabaseEventStatus::NotExistent
                },
            )
        }
    }

    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: &Timestamp,
    ) -> Result<bool, DatabaseError> {
        Ok(self
            .helper
            .has_coordinate_been_deleted(coordinate, timestamp)
            .await)
    }

    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: RelayUrl,
    ) -> Result<(), DatabaseError> {
        let mut set: HashSet<RelayUrl> = self
            .event_seen_on_relays(&event_id)
            .await?
            .unwrap_or_else(|| HashSet::with_capacity(1));

        if set.insert(relay_url) {
            let tx = self
                .db
                .transaction_on_one_with_mode(
                    EVENTS_SEEN_BY_RELAYS_CF,
                    IdbTransactionMode::Readwrite,
                )
                .map_err(into_err)?;
            let store = tx
                .object_store(EVENTS_SEEN_BY_RELAYS_CF)
                .map_err(into_err)?;
            let key = JsValue::from(event_id.to_hex());

            // Acquire FlatBuffers Builder
            let mut fbb = self.fbb.lock().await;

            // Encode
            let value = JsValue::from(hex::encode(set.encode(&mut fbb)));

            // Drop FlatBuffers Builder
            drop(fbb);

            // Save
            store
                .put_key_val(&key, &value)
                .map_err(into_err)?
                .await
                .map_err(into_err)?;
        }

        Ok(())
    }

    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(EVENTS_SEEN_BY_RELAYS_CF, IdbTransactionMode::Readonly)
            .map_err(into_err)?;
        let store = tx
            .object_store(EVENTS_SEEN_BY_RELAYS_CF)
            .map_err(into_err)?;
        let key = JsValue::from(event_id.to_hex());
        if let Some(jsvalue) = store.get(&key).map_err(into_err)?.await.map_err(into_err)? {
            if let Some(set_hex) = js_value_to_string(jsvalue) {
                let bytes = hex::decode(set_hex).map_err(DatabaseError::backend)?;
                return Ok(Some(
                    HashSet::decode(&bytes).map_err(DatabaseError::backend)?,
                ));
            }
        }

        Ok(None)
    }

    async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Event>, DatabaseError> {
        Ok(self.helper.event_by_id(event_id).await)
    }

    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        Ok(self.helper.count(filters).await)
    }

    async fn query(&self, filters: Vec<Filter>) -> Result<Events, DatabaseError> {
        Ok(self.helper.query(filters).await)
    }

    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        Ok(self.helper.negentropy_items(filter).await)
    }

    async fn delete(&self, filter: Filter) -> Result<(), DatabaseError> {
        self._delete(filter).await.map_err(DatabaseError::backend)
    }
});

fn js_value_to_string(value: JsValue) -> Option<String> {
    let s: JsString = value.dyn_into().ok()?;
    Some(s.into())
}
