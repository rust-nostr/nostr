// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;

use nostr::url::Url;
use nostr::SubscriptionFilter;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Subscription {
    filters: Vec<SubscriptionFilter>,
    channels: HashMap<Url, Channel>,
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

    /// Update subscription filters
    pub fn update_filters(&mut self, filters: Vec<SubscriptionFilter>) {
        self.filters = filters;
    }

    /// Get subscription filters
    pub fn get_filters(&self) -> Vec<SubscriptionFilter> {
        self.filters.clone()
    }

    /// Add new subscription channel
    pub fn add_channel(&mut self, relay_url: &Url, channel: Channel) {
        self.channels.insert(relay_url.clone(), channel);
    }

    /// Remove subscription channel
    pub fn remove_channel(&mut self, relay_url: &Url) -> Option<Channel> {
        self.channels.remove(relay_url)
    }

    /// Get subscription channels
    pub fn get_channel(&mut self, relay_url: &Url) -> Channel {
        self.channels
            .entry(relay_url.clone())
            .or_insert_with(|| Channel::new(relay_url.clone()))
            .clone()
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    id: Uuid,
    relay_url: Url,
}

impl Channel {
    /// Create new subscription channel
    pub fn new(relay_url: Url) -> Self {
        Self {
            id: Uuid::new_v4(),
            relay_url,
        }
    }

    /// Get channel id
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get channel relay url
    pub fn relay_url(&self) -> Url {
        self.relay_url.clone()
    }
}
