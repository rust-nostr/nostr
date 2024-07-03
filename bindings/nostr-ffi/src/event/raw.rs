// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::event::raw;
use nostr::JsonUtil;
use uniffi::{Object, Record};

use crate::error::Result;

/// Raw event
#[derive(Record, o2o::o2o)]
#[map_owned(raw::RawEvent)]
pub struct RawEventRecord {
    /// ID
    pub id: String,
    /// Author
    pub pubkey: String,
    /// Timestamp (seconds)
    pub created_at: u64,
    /// Kind
    pub kind: u16,
    /// Vector of strings
    pub tags: Vec<Vec<String>>,
    /// Content
    pub content: String,
    /// Signature
    pub sig: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct RawEvent {
    inner: raw::RawEvent,
}

impl From<raw::RawEvent> for RawEvent {
    fn from(inner: raw::RawEvent) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl RawEvent {
    #[uniffi::constructor]
    pub fn from_record(r: RawEventRecord) -> Self {
        Self { inner: r.into() }
    }

    #[uniffi::constructor]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(raw::RawEvent::from_json(json)?.into())
    }

    pub fn as_record(&self) -> RawEventRecord {
        self.inner.clone().into()
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }
}
