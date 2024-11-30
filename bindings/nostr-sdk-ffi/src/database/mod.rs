// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::prelude::{self, IntoNostrDatabase, NostrEventsDatabaseExt};
#[cfg(feature = "ndb")]
use nostr_sdk::NdbDatabase;
#[cfg(feature = "lmdb")]
use nostr_sdk::NostrLMDB;
use uniffi::Object;

pub mod events;

use self::events::Events;
use crate::error::Result;
use crate::protocol::{Event, EventId, Filter, Metadata, PublicKey};

#[derive(Object)]
pub struct NostrDatabase {
    inner: Arc<dyn prelude::NostrDatabase>,
}

impl Deref for NostrDatabase {
    type Target = Arc<dyn prelude::NostrDatabase>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Arc<dyn prelude::NostrDatabase>> for NostrDatabase {
    fn from(inner: Arc<dyn prelude::NostrDatabase>) -> Self {
        Self { inner }
    }
}

#[cfg(feature = "lmdb")]
#[uniffi::export]
impl NostrDatabase {
    /// LMDB backend
    #[uniffi::constructor]
    pub fn lmdb(path: &str) -> Result<Self> {
        let db = Arc::new(NostrLMDB::open(path)?);
        Ok(Self {
            inner: db.into_nostr_database(),
        })
    }
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
    // TODO: revert 4da38c9406f8552eef48ffe7ed4486ddc52392a6
    // TODO: re-allow to use custom database (only for events)?

    /// Save [`Event`] into store
    ///
    /// Return `true` if event was successfully saved into database.
    pub async fn save_event(&self, event: &Event) -> Result<bool> {
        Ok(self.inner.save_event(event.deref()).await?)
    }

    /// Get list of relays that have seen the [`EventId`]
    pub async fn event_seen_on_relays(&self, event_id: &EventId) -> Result<Option<Vec<String>>> {
        let res = self.inner.event_seen_on_relays(event_id.deref()).await?;
        Ok(res.map(|set| set.into_iter().map(|u| u.to_string()).collect()))
    }

    /// Get [`Event`] by [`EventId`]
    pub async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Arc<Event>>> {
        Ok(self
            .inner
            .event_by_id(event_id.deref())
            .await?
            .map(|e| Arc::new(e.into())))
    }

    pub async fn count(&self, filters: Vec<Arc<Filter>>) -> Result<u64> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self.inner.count(filters).await? as u64)
    }

    pub async fn query(&self, filters: Vec<Arc<Filter>>) -> Result<Arc<Events>> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(Arc::new(self.inner.query(filters).await?.into()))
    }

    /// Delete all events that match the `Filter`
    pub async fn delete(&self, filter: &Filter) -> Result<()> {
        Ok(self.inner.delete(filter.deref().clone()).await?)
    }

    /// Wipe all data
    pub async fn wipe(&self) -> Result<()> {
        Ok(self.inner.wipe().await?)
    }

    pub async fn metadata(&self, public_key: &PublicKey) -> Result<Option<Arc<Metadata>>> {
        Ok(self
            .inner
            .metadata(**public_key)
            .await?
            .map(|m| Arc::new(m.into())))
    }
}
