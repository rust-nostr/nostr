// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use js_sys::Array;
use nostr_sdk::database::{DynNostrDatabase, IntoNostrDatabase, NostrDatabaseExt};
use nostr_sdk::WebDatabase;
use wasm_bindgen::prelude::*;

pub mod events;

pub use self::events::JsEvents;
use crate::error::{into_err, Result};
use crate::profile::JsProfile;
use crate::protocol::event::{JsEvent, JsEventId};
use crate::protocol::key::JsPublicKey;
use crate::protocol::types::JsFilter;
use crate::JsStringArray;

/// Nostr Database
#[wasm_bindgen(js_name = NostrDatabase)]
pub struct JsNostrDatabase {
    inner: Arc<DynNostrDatabase>,
}

impl From<Arc<DynNostrDatabase>> for JsNostrDatabase {
    fn from(inner: Arc<DynNostrDatabase>) -> Self {
        Self { inner }
    }
}

impl From<&JsNostrDatabase> for Arc<DynNostrDatabase> {
    fn from(db: &JsNostrDatabase) -> Self {
        db.inner.clone()
    }
}

#[wasm_bindgen(js_class = NostrDatabase)]
impl JsNostrDatabase {
    /// Open/Create database with **unlimited** capacity
    pub async fn indexeddb(name: &str) -> Result<JsNostrDatabase> {
        let db = Arc::new(WebDatabase::open(name).await.map_err(into_err)?);
        Ok(Self {
            inner: db.into_nostr_database(),
        })
    }

    /// Open/Create database with **limited** capacity
    #[wasm_bindgen(js_name = indexeddbBounded)]
    pub async fn indexeddb_bounded(name: &str, max_capacity: u64) -> Result<JsNostrDatabase> {
        let db = Arc::new(
            WebDatabase::open_bounded(name, max_capacity as usize)
                .await
                .map_err(into_err)?,
        );
        Ok(Self {
            inner: db.into_nostr_database(),
        })
    }

    /// Save `Event` into store
    ///
    /// Return `true` if event was successfully saved into database.
    ///
    /// **This method assume that `Event` was already verified**
    pub async fn save_event(&self, event: &JsEvent) -> Result<bool> {
        self.inner.save_event(event).await.map_err(into_err)
    }
    /// Get list of relays that have seen the [`EventId`]
    #[wasm_bindgen(js_name = eventSeenOnRelays)]
    pub async fn event_seen_on_relays(
        &self,
        event_id: &JsEventId,
    ) -> Result<Option<JsStringArray>> {
        let res = self
            .inner
            .event_seen_on_relays(event_id.deref())
            .await
            .map_err(into_err)?;
        Ok(res.map(|set| {
            set.into_iter()
                .map(|u| JsValue::from(u.to_string()))
                .collect::<Array>()
                .unchecked_into()
        }))
    }

    /// Get [`Event`] by [`EventId`]
    #[wasm_bindgen(js_name = eventById)]
    pub async fn event_by_id(&self, event_id: &JsEventId) -> Result<Option<JsEvent>> {
        Ok(self
            .inner
            .event_by_id(event_id.deref())
            .await
            .map_err(into_err)?
            .map(|e| e.into()))
    }

    pub async fn count(&self, filters: Vec<JsFilter>) -> Result<u64> {
        let filters = filters.into_iter().map(|f| f.into()).collect();
        Ok(self.inner.count(filters).await.map_err(into_err)? as u64)
    }

    pub async fn query(&self, filters: Vec<JsFilter>) -> Result<JsEvents> {
        let filters = filters.into_iter().map(|f| f.into()).collect();
        Ok(self.inner.query(filters).await.map_err(into_err)?.into())
    }

    /// Wipe all data
    pub async fn wipe(&self) -> Result<()> {
        self.inner.wipe().await.map_err(into_err)
    }

    pub async fn profile(&self, public_key: &JsPublicKey) -> Result<JsProfile> {
        Ok(self
            .inner
            .profile(**public_key)
            .await
            .map_err(into_err)?
            .into())
    }
}
