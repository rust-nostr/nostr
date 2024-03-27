// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::{Event, EventId, Filter, PublicKey};
use nostr_sdk::database::{DynNostrDatabase, IntoNostrDatabase, NostrDatabaseExt, Order};
#[cfg(feature = "ndb")]
use nostr_sdk::NdbDatabase;
#[cfg(feature = "sqlite")]
use nostr_sdk::SQLiteDatabase;
use uniffi::Object;

pub mod custom;

use self::custom::{CustomNostrDatabase, IntermediateCustomNostrDatabase};
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

#[cfg(feature = "sqlite")]
#[uniffi::export(async_runtime = "tokio")]
impl NostrDatabase {
    #[uniffi::constructor]
    pub async fn sqlite(path: &str) -> Result<Self> {
        let db = Arc::new(SQLiteDatabase::open(path).await?);
        Ok(Self {
            inner: db.into_nostr_database(),
        })
    }

    // `#[uniffi::export(async_runtime = "tokio")]` require a method
    async fn _none(&self) {}
}

#[cfg(feature = "ndb")]
#[uniffi::export]
impl NostrDatabase {
    /// [`nostrdb`](https://github.com/damus-io/nostrdb) backend
    #[uniffi::constructor]
    pub fn ndb(path: &str) -> Result<Self> {
        let db = Arc::new(NdbDatabase::open(path)?);
        Ok(Self {
            inner: db.into_nostr_database(),
        })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl NostrDatabase {
    #[uniffi::constructor]
    pub fn custom(database: Arc<dyn CustomNostrDatabase>) -> Self {
        let intermediate = IntermediateCustomNostrDatabase { inner: database };

        Self {
            inner: intermediate.into_nostr_database(),
        }
    }

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    pub async fn save_event(&self, event: &Event) -> Result<bool> {
        Ok(self.inner.save_event(event.deref()).await?)
    }

    /// Get list of relays that have seen the [`EventId`]
    pub async fn event_seen_on_relays(&self, event_id: &EventId) -> Result<Option<Vec<String>>> {
        let res = self.inner.event_seen_on_relays(**event_id).await?;
        Ok(res.map(|set| set.into_iter().map(|u| u.to_string()).collect()))
    }

    /// Get [`Event`] by [`EventId`]
    pub async fn event_by_id(&self, event_id: &EventId) -> Result<Arc<Event>> {
        Ok(Arc::new(self.inner.event_by_id(**event_id).await?.into()))
    }

    pub async fn count(&self, filters: Vec<Arc<Filter>>) -> Result<u64> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self.inner.count(filters).await? as u64)
    }

    pub async fn query(&self, filters: Vec<Arc<Filter>>) -> Result<Vec<Arc<Event>>> {
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
    }

    /// Delete all events that match the `Filter`
    pub async fn delete(&self, filter: &Filter) -> Result<()> {
        Ok(self.inner.delete(filter.deref().clone()).await?)
    }

    /// Wipe all data
    pub async fn wipe(&self) -> Result<()> {
        Ok(self.inner.wipe().await?)
    }

    pub async fn profile(&self, public_key: &PublicKey) -> Result<Arc<Profile>> {
        Ok(Arc::new(self.inner.profile(**public_key).await?.into()))
    }
}
