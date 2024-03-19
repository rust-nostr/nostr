// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::Arc;

use js_sys::Array;
use nostr_js::error::{into_err, Result};
use nostr_js::event::{JsEvent, JsEventArray, JsEventId};
use nostr_js::key::JsPublicKey;
use nostr_js::message::JsFilter;
use nostr_js::JsStringArray;
use nostr_sdk::database::{DynNostrDatabase, IntoNostrDatabase, NostrDatabaseExt, Order};
use nostr_sdk::WebDatabase;
use wasm_bindgen::prelude::*;

use crate::profile::JsProfile;

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
    /// Open IndexedDB database
    ///
    /// If not exists, create it.
    pub async fn indexeddb(name: String) -> Result<JsNostrDatabase> {
        let db = Arc::new(WebDatabase::open(name).await.map_err(into_err)?);
        Ok(Self {
            inner: db.into_nostr_database(),
        })
    }

    // /// Save [`Event`] into store
    //
    // Return `true` if event was successfully saved into database.
    // pub fn save_event(&self, event: &JsEvent) -> Result<bool> {
    // block_on(async move { Ok(self.inner.save_event(event.as_ref().deref()).await?) })
    // }

    /// Get list of relays that have seen the [`EventId`]
    #[wasm_bindgen(js_name = eventSeenOnRelays)]
    pub async fn event_seen_on_relays(
        &self,
        event_id: &JsEventId,
    ) -> Result<Option<JsStringArray>> {
        let res = self
            .inner
            .event_seen_on_relays(**event_id)
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
    pub async fn event_by_id(&self, event_id: &JsEventId) -> Result<JsEvent> {
        Ok(self
            .inner
            .event_by_id(**event_id)
            .await
            .map_err(into_err)?
            .into())
    }

    pub async fn count(&self, filters: Vec<JsFilter>) -> Result<u64> {
        let filters = filters.into_iter().map(|f| f.into()).collect();
        Ok(self.inner.count(filters).await.map_err(into_err)? as u64)
    }

    pub async fn query(&self, filters: Vec<JsFilter>) -> Result<JsEventArray> {
        let filters = filters.into_iter().map(|f| f.into()).collect();
        Ok(self
            .inner
            .query(filters, Order::Desc)
            .await
            .map_err(into_err)?
            .into_iter()
            .map(|e| {
                let event: JsEvent = e.into();
                JsValue::from(event)
            })
            .collect::<Array>()
            .unchecked_into())
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
