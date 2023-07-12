// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::secp256k1::XOnlyPublicKey;
use nostr::{EventId, Filter as FilterSdk, Timestamp};

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;

#[derive(Clone)]
pub struct Filter {
    sub_filter: FilterSdk,
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Filter {
    type Target = FilterSdk;
    fn deref(&self) -> &Self::Target {
        &self.sub_filter
    }
}

impl From<FilterSdk> for Filter {
    fn from(f: FilterSdk) -> Self {
        Self { sub_filter: f }
    }
}

impl Filter {
    pub fn new() -> Self {
        Self {
            sub_filter: FilterSdk::new(),
        }
    }

    pub fn id(self: Arc<Self>, id: String) -> Result<Arc<Self>> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.id(id);

        Ok(Arc::new(builder))
    }

    pub fn ids(self: Arc<Self>, ids: Vec<String>) -> Result<Arc<Self>> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.ids(ids);

        Ok(Arc::new(builder))
    }

    pub fn kind(self: Arc<Self>, kind: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.kind(kind.into());

        Arc::new(builder)
    }

    pub fn event(self: Arc<Self>, event_id: String) -> Result<Arc<Self>> {
        let event_id = EventId::from_hex(event_id)?;

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.events(vec![event_id]);

        Ok(Arc::new(builder))
    }

    pub fn pubkey(self: Arc<Self>, pubkey: String) -> Result<Arc<Self>> {
        let pubkey = XOnlyPublicKey::from_str(&pubkey)?;

        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.pubkey(pubkey);

        Ok(Arc::new(builder))
    }

    pub fn since(self: Arc<Self>, timestamp: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.since(Timestamp::from(timestamp));

        Arc::new(builder)
    }

    pub fn until(self: Arc<Self>, timestamp: u64) -> Arc<Self> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.until(Timestamp::from(timestamp));

        Arc::new(builder)
    }

    pub fn authors(self: Arc<Self>, authors: Vec<String>) -> Result<Arc<Self>> {
        let mut builder = unwrap_or_clone_arc(self);
        builder.sub_filter = builder.sub_filter.authors(authors);

        Ok(Arc::new(builder))
    }
}
