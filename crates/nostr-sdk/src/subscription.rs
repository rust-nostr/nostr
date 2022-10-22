// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;

use nostr_sdk_base::SubscriptionFilter;
use uuid::Uuid;

#[derive(Clone)]
pub struct Subscription {
    filters: Vec<SubscriptionFilter>,
    channels: HashMap<String, Channel>,
}

impl Default for Subscription {
    fn default() -> Self {
        Self::new()
    }
}

impl Subscription {
    pub fn new() -> Self {
        Self {
            filters: vec![],
            channels: HashMap::new(),
        }
    }

    pub fn update_filters(&mut self, filters: Vec<SubscriptionFilter>) {
        self.filters = filters;
    }

    pub fn get_filters(&self) -> Vec<SubscriptionFilter> {
        self.filters.clone()
    }

    pub fn add_channel(&mut self, relay_url: String, channel: Channel) {
        self.channels.insert(relay_url, channel);
    }

    pub fn remove_channel(&mut self, relay_url: &str) -> Option<Channel> {
        self.channels.remove(relay_url)
    }

    pub fn get_channel(&mut self, relay_url: &str) -> Channel {
        self.channels
            .entry(relay_url.into())
            .or_insert_with(|| Channel::new(relay_url))
            .clone()
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: Uuid,
    pub relay_url: String,
}

impl Channel {
    pub fn new(relay_url: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            relay_url: relay_url.into(),
        }
    }
}
