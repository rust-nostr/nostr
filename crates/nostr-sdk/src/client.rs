// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::str::FromStr;

use anyhow::Result;
use bitcoin_hashes::sha256::Hash;
use nostr_sdk_base::{Event, Keys, SubscriptionFilter};
use tokio::sync::broadcast;

use crate::relay::{RelayPool, RelayPoolNotifications};
#[cfg(feature = "blocking")]
use crate::RUNTIME;

pub struct Client {
    pub pool: RelayPool,
    pub keys: Keys,
}

impl Client {
    pub fn new(keys: &Keys) -> Self {
        Self {
            pool: RelayPool::new(),
            keys: keys.clone(),
        }
    }

    pub fn generate_keys() -> Keys {
        Keys::generate_from_os_random()
    }

    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotifications> {
        self.pool.notifications()
    }

    pub fn add_relay(&mut self, url: &str, proxy: Option<SocketAddr>) -> Result<()> {
        self.pool.add_relay(url, proxy)
    }
}

#[cfg(not(feature = "blocking"))]
impl Client {
    pub async fn remove_relay(&mut self, url: &str) -> Result<()> {
        self.pool.remove_relay(url).await
    }

    pub async fn connect_relay(&mut self, url: &str) -> Result<()> {
        self.pool.connect_relay(url).await
    }

    pub async fn disconnect_relay(&mut self, url: &str) -> Result<()> {
        self.pool.disconnect_relay(url).await
    }

    /// Connect to all disconnected relays
    pub async fn connect(&mut self) -> Result<()> {
        self.pool.connect().await
    }

    pub async fn subscribe(&mut self, filters: Vec<SubscriptionFilter>) -> Result<()> {
        self.pool.subscribe(filters).await
    }

    pub async fn send_event(&self, event: Event) -> Result<()> {
        self.pool.send_event(event).await
    }

    pub async fn delete_event(&self, event_id: &str) -> Result<()> {
        let event = Event::delete(&self.keys, vec![Hash::from_str(event_id)?], None)?;
        self.send_event(event).await
    }

    pub async fn handle_notifications<F>(&self, func: F) -> Result<()>
    where
        F: Fn(RelayPoolNotifications) -> Result<()>,
    {
        loop {
            let mut notifications = self.notifications();

            while let Ok(notification) = notifications.recv().await {
                func(notification)?;
            }
        }
    }
}

#[cfg(feature = "blocking")]
impl Client {
    pub fn remove_relay(&mut self, url: &str) -> Result<()> {
        RUNTIME.block_on(async { self.pool.remove_relay(url).await })
    }

    pub fn connect_relay(&mut self, url: &str) -> Result<()> {
        RUNTIME.block_on(async { self.pool.connect_relay(url).await })
    }

    pub fn disconnect_relay(&mut self, url: &str) -> Result<()> {
        RUNTIME.block_on(async { self.pool.disconnect_relay(url).await })
    }

    /// Connect to all disconnected relays
    pub fn connect(&mut self) -> Result<()> {
        RUNTIME.block_on(async { self.pool.connect().await })
    }

    pub fn subscribe(&mut self, filters: Vec<SubscriptionFilter>) -> Result<()> {
        RUNTIME.block_on(async { self.pool.subscribe(filters).await })
    }

    pub fn send_event(&self, event: Event) -> Result<()> {
        RUNTIME.block_on(async { self.pool.send_event(event).await })
    }

    pub fn delete_event(&self, event_id: &str) -> Result<()> {
        let event = Event::delete(&self.keys, vec![Hash::from_str(event_id)?], None)?;
        self.send_event(event)
    }

    pub fn handle_notifications<F>(&self, func: F) -> Result<()>
    where
        F: Fn(RelayPoolNotifications) -> Result<()>,
    {
        RUNTIME.block_on(async {
            loop {
                let mut notifications = self.notifications();

                while let Ok(notification) = notifications.recv().await {
                    func(notification)?;
                }
            }
        })
    }
}
