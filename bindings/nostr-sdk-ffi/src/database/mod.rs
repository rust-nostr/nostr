// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr_sdk::prelude::{self, IntoNostrDatabase};
#[cfg(feature = "ndb")]
use nostr_sdk::NdbDatabase;
#[cfg(feature = "lmdb")]
use nostr_sdk::NostrLMDB;
use uniffi::{Enum, Object};

pub mod events;

use self::events::Events;
use crate::error::Result;
use crate::protocol::event::Event;
use crate::protocol::filter::Filter;

/// Reason why event wasn't stored into the database
#[derive(Enum)]
pub enum RejectedReason {
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

impl From<prelude::RejectedReason> for RejectedReason {
    fn from(status: prelude::RejectedReason) -> Self {
        match status {
            prelude::RejectedReason::Ephemeral => Self::Ephemeral,
            prelude::RejectedReason::Duplicate => Self::Duplicate,
            prelude::RejectedReason::Deleted => Self::Deleted,
            prelude::RejectedReason::Expired => Self::Expired,
            prelude::RejectedReason::Replaced => Self::Replaced,
            prelude::RejectedReason::InvalidDelete => Self::InvalidDelete,
            prelude::RejectedReason::Other => Self::Other,
        }
    }
}

/// Save event status
#[derive(Enum)]
pub enum SaveEventStatus {
    /// The event has been successfully saved
    Success,
    /// The event has been rejected
    Rejected(RejectedReason),
}

impl From<prelude::SaveEventStatus> for SaveEventStatus {
    fn from(status: prelude::SaveEventStatus) -> Self {
        match status {
            prelude::SaveEventStatus::Success => Self::Success,
            prelude::SaveEventStatus::Rejected(reason) => Self::Rejected(reason.into()),
        }
    }
}

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
    /// Save [`Event`] into store
    pub async fn save_event(&self, event: &Event) -> Result<SaveEventStatus> {
        Ok(self.inner.save_event(event.deref()).await?.into())
    }

    pub async fn count(&self, filters: Vec<Arc<Filter>>) -> Result<u64> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self.inner.count(filters).await? as u64)
    }

    pub async fn query(&self, filters: Vec<Arc<Filter>>) -> Result<Events> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self.inner.query(filters).await?.into())
    }

    /// Delete all events that match the `Filter`
    pub async fn delete(&self, filter: &Filter) -> Result<()> {
        Ok(self.inner.delete(filter.deref().clone()).await?)
    }

    /// Wipe all data
    pub async fn wipe(&self) -> Result<()> {
        Ok(self.inner.wipe().await?)
    }
}
