// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::url::Url;
use nostr_ffi::SubscriptionFilter;
use nostr_sdk::subscription::{Channel as ChannelSdk, Subscription as SubscriptionSdk};
use parking_lot::RwLock;

use crate::error::Result;

pub struct Subscription {
    sub: Arc<RwLock<SubscriptionSdk>>,
}

impl Default for Subscription {
    fn default() -> Self {
        Self::new()
    }
}

impl Subscription {
    pub fn new() -> Self {
        Self {
            sub: Arc::new(RwLock::new(SubscriptionSdk::new())),
        }
    }

    pub fn update_filters(&self, filters: Vec<Arc<SubscriptionFilter>>) {
        let mut new_filters: Vec<nostr::SubscriptionFilter> = Vec::with_capacity(filters.len());
        for filter in filters.into_iter() {
            new_filters.push(filter.as_ref().deref().clone());
        }

        let mut sub = self.sub.write();
        sub.update_filters(new_filters);
    }

    pub fn get_filters(&self) -> Vec<Arc<SubscriptionFilter>> {
        let sub = self.sub.read();
        sub.get_filters()
            .into_iter()
            .map(|s| Arc::new(SubscriptionFilter::from(s)))
            .collect()
    }

    pub fn add_channel(&self, relay_url: String, channel: Arc<Channel>) -> Result<()> {
        let relay_url = Url::parse(&relay_url)?;
        let mut sub = self.sub.write();
        sub.add_channel(&relay_url, channel.as_ref().deref().clone());
        Ok(())
    }

    pub fn remove_channel(&self, relay_url: String) -> Result<Option<Arc<Channel>>> {
        let relay_url = Url::parse(&relay_url)?;
        let mut sub = self.sub.write();
        Ok(sub.remove_channel(&relay_url).map(|ch| Arc::new(ch.into())))
    }

    pub fn get_channel(&self, relay_url: String) -> Result<Arc<Channel>> {
        let relay_url = Url::parse(&relay_url)?;
        let mut sub = self.sub.write();
        Ok(Arc::new(sub.get_channel(&relay_url).into()))
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    ch: ChannelSdk,
}

impl Deref for Channel {
    type Target = ChannelSdk;
    fn deref(&self) -> &Self::Target {
        &self.ch
    }
}

impl From<ChannelSdk> for Channel {
    fn from(ch: ChannelSdk) -> Self {
        Self { ch }
    }
}

impl Channel {
    pub fn new(relay_url: String) -> Result<Self> {
        let relay_url = Url::parse(&relay_url)?;
        Ok(Self {
            ch: ChannelSdk::new(relay_url),
        })
    }

    pub fn id(&self) -> String {
        self.ch.id().to_string()
    }

    pub fn relay_url(&self) -> String {
        self.ch.relay_url().to_string()
    }
}
