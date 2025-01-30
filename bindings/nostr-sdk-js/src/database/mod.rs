// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod events;

pub use self::events::JsEvents;
use crate::error::{into_err, Result};
use crate::protocol::event::{JsEvent, JsEventId};
use crate::protocol::filter::JsFilter;
use crate::protocol::key::JsPublicKey;
use crate::protocol::nips::nip01::JsMetadata;

#[wasm_bindgen(js_name = SaveEventStatus)]
pub enum JsSaveEventStatus {
    /// The event has been successfully saved into the database
    Success,
    /// Ephemeral events aren't expected to be stored
    Ephemeral,
    /// The event already exists
    Duplicate,
    /// The event was deleted
    Deleted,
    /// The event is expired
    Expired,
    /// The event was replaced
    Replaced,
    /// Attempt to delete a non-owned event
    InvalidDelete,
    /// Other reason
    Other,
}

impl From<SaveEventStatus> for JsSaveEventStatus {
    fn from(status: SaveEventStatus) -> Self {
        match status {
            SaveEventStatus::Success => Self::Success,
            SaveEventStatus::Rejected(RejectedReason::Ephemeral) => Self::Ephemeral,
            SaveEventStatus::Rejected(RejectedReason::Duplicate) => Self::Duplicate,
            SaveEventStatus::Rejected(RejectedReason::Deleted) => Self::Deleted,
            SaveEventStatus::Rejected(RejectedReason::Expired) => Self::Expired,
            SaveEventStatus::Rejected(RejectedReason::Replaced) => Self::Replaced,
            SaveEventStatus::Rejected(RejectedReason::InvalidDelete) => Self::InvalidDelete,
            SaveEventStatus::Rejected(RejectedReason::Other) => Self::Other,
        }
    }
}

/// Nostr Database
#[wasm_bindgen(js_name = NostrDatabase)]
pub struct JsNostrDatabase {
    inner: Arc<dyn NostrDatabase>,
}

impl Deref for JsNostrDatabase {
    type Target = Arc<dyn NostrDatabase>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Arc<dyn NostrDatabase>> for JsNostrDatabase {
    fn from(inner: Arc<dyn NostrDatabase>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NostrDatabase)]
impl JsNostrDatabase {
    /// Open/Create database with **unlimited** capacity
    pub async fn indexeddb(name: &str) -> Result<JsNostrDatabase> {
        let db = WebDatabase::open(name).await.map_err(into_err)?;
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
    /// **This method assumes that `Event` was already verified**
    pub async fn save_event(&self, event: &JsEvent) -> Result<JsSaveEventStatus> {
        Ok(self.inner.save_event(event).await.map_err(into_err)?.into())
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

    pub async fn count(&self, filter: &JsFilter) -> Result<u64> {
        Ok(self
            .inner
            .count(filter.deref().clone())
            .await
            .map_err(into_err)? as u64)
    }

    pub async fn query(&self, filter: &JsFilter) -> Result<JsEvents> {
        Ok(self
            .inner
            .query(filter.deref().clone())
            .await
            .map_err(into_err)?
            .into())
    }

    /// Wipe all data
    pub async fn wipe(&self) -> Result<()> {
        self.inner.wipe().await.map_err(into_err)
    }

    pub async fn metadata(&self, public_key: &JsPublicKey) -> Result<Option<JsMetadata>> {
        Ok(self
            .inner
            .metadata(**public_key)
            .await
            .map_err(into_err)?
            .map(|m| m.into()))
    }
}
