// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::{Event, EventId, Filter, PublicKey};
use nostr_sdk::database::{DynNostrDatabase, IntoNostrDatabase, NostrDatabaseExt, Order};
use nostr_sdk::{block_on, SQLiteDatabase};
use uniffi::Object;

use crate::error::Result;
use crate::profile::Profile;

#[derive(Object)]
pub struct NostrDatabase {
    inner: Arc<DynNostrDatabase>,
}

impl From<Arc<DynNostrDatabase>> for NostrDatabase {
    fn from(inner: Arc<DynNostrDatabase>) -> Self {
        Self { inner }
    }
}

impl From<&NostrDatabase> for Arc<DynNostrDatabase> {
    fn from(db: &NostrDatabase) -> Self {
        db.inner.clone()
    }
}

#[uniffi::export]
impl NostrDatabase {
    #[uniffi::constructor]
    pub fn sqlite(path: String) -> Result<Arc<Self>> {
        block_on(async move {
            let db = Arc::new(SQLiteDatabase::open(path).await?);
            Ok(Arc::new(Self {
                inner: db.into_nostr_database(),
            }))
        })
    }

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    pub fn save_event(&self, event: Arc<Event>) -> Result<bool> {
        block_on(async move { Ok(self.inner.save_event(event.as_ref().deref()).await?) })
    }

    /// Get list of relays that have seen the [`EventId`]
    pub fn event_seen_on_relays(&self, event_id: Arc<EventId>) -> Result<Option<Vec<String>>> {
        block_on(async move {
            let res = self.inner.event_seen_on_relays(**event_id).await?;
            Ok(res.map(|set| set.into_iter().map(|u| u.to_string()).collect()))
        })
    }

    /// Get [`Event`] by [`EventId`]
    pub fn event_by_id(&self, event_id: Arc<EventId>) -> Result<Arc<Event>> {
        block_on(async move { Ok(Arc::new(self.inner.event_by_id(**event_id).await?.into())) })
    }

    pub fn count(&self, filters: Vec<Arc<Filter>>) -> Result<u64> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        block_on(async move { Ok(self.inner.count(filters).await? as u64) })
    }

    pub fn query(&self, filters: Vec<Arc<Filter>>) -> Result<Vec<Arc<Event>>> {
        block_on(async move {
            let filters = filters
                .into_iter()
                .map(|f| f.as_ref().deref().clone())
                .collect();
            Ok(self
                .inner
                .query(filters, Order::Desc)
                .await?
                .into_iter()
                .map(|e| Arc::new(e.into()))
                .collect())
        })
    }

    /// Wipe all data
    pub fn wipe(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.wipe().await?) })
    }

    pub fn profile(&self, public_key: Arc<PublicKey>) -> Result<Arc<Profile>> {
        block_on(async move { Ok(Arc::new(self.inner.profile(**public_key).await?.into())) })
    }
}
