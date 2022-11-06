// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use bitcoin_hashes::sha256::Hash;
use crossbeam_channel::{select, Receiver};
use nostr_sdk_base::{Contact, Event, Keys, SubscriptionFilter};
use nostr_sdk_common::thread;
use tokio::sync::Mutex;

use crate::relay::{RelayPool, RelayPoolNotifications};
#[cfg(feature = "blocking")]
use crate::RUNTIME;

#[derive(Clone)]
pub struct Client {
    pub pool: Arc<Mutex<RelayPool>>,
    pub keys: Keys,
    pub contacts: Arc<Mutex<Vec<Contact>>>,
}

impl Client {
    pub fn new(keys: &Keys, contacts: Option<Vec<Contact>>) -> Self {
        Self {
            pool: Arc::new(Mutex::new(RelayPool::new())),
            keys: keys.clone(),
            contacts: Arc::new(Mutex::new(contacts.unwrap_or_default())),
        }
    }

    pub fn generate_keys() -> Keys {
        Keys::generate_from_os_random()
    }
}

#[cfg(not(feature = "blocking"))]
impl Client {
    pub async fn add_contact(&self, contact: Contact) {
        let mut contacts = self.contacts.lock().await;
        if !contacts.contains(&contact) {
            contacts.push(contact);
        }
    }

    pub async fn remove_contact(&self, contact: &Contact) {
        let mut contacts = self.contacts.lock().await;
        if contacts.contains(contact) {
            contacts.retain(|c| c != contact);
        }
    }

    pub async fn notifications(&self) -> Receiver<RelayPoolNotifications> {
        let pool = self.pool.lock().await;
        pool.notifications()
    }

    pub async fn add_relay(&self, url: &str) -> Result<()> {
        let mut pool = self.pool.lock().await;
        pool.add_relay(url)?;
        Ok(())
    }

    pub async fn remove_relay(&self, url: &str) -> Result<()> {
        let mut pool = self.pool.lock().await;
        pool.remove_relay(url).await?;
        Ok(())
    }

    pub async fn connect_relay(&self, url: &str) {
        let mut pool = self.pool.lock().await;
        pool.connect_relay(url).await;
    }

    pub async fn disconnect_relay(&self, url: &str) {
        let mut pool = self.pool.lock().await;
        pool.disconnect_relay(url).await;
    }

    /// Connect to all disconnected relays
    pub async fn connect_all(&self) {
        let mut pool = self.pool.lock().await;
        pool.connect_all().await;
    }

    /// Connect to all relays and every 60 sec try to reconnect to disconnected ones
    pub async fn connect_and_keep_alive(&self) {
        let client = self.clone();
        client.connect_all().await;
        tokio::spawn(async move {
            loop {
                thread::sleep(60);
                client.connect_all().await;
            }
        });
    }

    pub async fn subscribe(&self, filters: Vec<SubscriptionFilter>) {
        let mut pool = self.pool.lock().await;
        pool.start_sub(filters).await;
    }

    pub async fn send_event(&self, event: Event) -> Result<()> {
        let pool = self.pool.lock().await;
        pool.send_event(event).await
    }

    pub async fn delete_event(&self, event_id: &str) -> Result<()> {
        let event = Event::delete(&self.keys, vec![Hash::from_str(event_id)?], None)?;
        self.send_event(event).await
    }

    pub async fn handle_notifications<F>(&self, func: F) -> Result<()>
    where
        F: Fn(RelayPoolNotifications) -> Result<()>,
    {
        let notifications = self.notifications().await;

        loop {
            select! {
                recv(notifications) -> result => {
                    if let Ok(notification) = result {
                        func(notification)?;
                    }
                }
            }
        }
    }
}

#[cfg(feature = "blocking")]
impl Client {
    pub fn add_contact(&self, contact: Contact) {
        RUNTIME.block_on(async {
            let mut contacts = self.contacts.lock().await;
            if !contacts.contains(&contact) {
                contacts.push(contact);
            }
        });
    }

    pub fn remove_contact(&self, contact: &Contact) {
        RUNTIME.block_on(async {
            let mut contacts = self.contacts.lock().await;
            if contacts.contains(contact) {
                contacts.retain(|c| c != contact);
            }
        });
    }

    pub fn notifications(&self) -> Receiver<RelayPoolNotifications> {
        RUNTIME.block_on(async {
            let pool = self.pool.lock().await;
            pool.notifications()
        })
    }

    pub fn add_relay(&self, url: &str) -> Result<()> {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.add_relay(url)?;
            Ok(())
        })
    }

    pub fn remove_relay(&self, url: &str) -> Result<()> {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.remove_relay(url).await?;
            Ok(())
        })
    }

    pub fn connect_relay(&self, url: &str) {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.connect_relay(url).await;
        });
    }

    pub fn disconnect_relay(&self, url: &str) {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.disconnect_relay(url).await;
        });
    }

    /// Connect to all disconnected relays
    pub fn connect_all(&self) {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.connect_all().await;
        });
    }

    /// Connect to all relays and every 60 sec try to reconnect to disconnected ones
    pub fn connect_and_keep_alive(&self) {
        let client = self.clone();
        client.connect_all();
        thread::spawn("connect_all_and_keep_alive", move || loop {
            thread::sleep(60);
            client.connect_all();
        });
    }

    pub fn subscribe(&self, filters: Vec<SubscriptionFilter>) {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.start_sub(filters).await;
        });
    }

    pub fn send_event(&self, event: Event) -> Result<()> {
        RUNTIME.block_on(async {
            let pool = self.pool.lock().await;
            pool.send_event(event).await
        })
    }

    pub fn delete_event(&self, event_id: &str) -> Result<()> {
        let event = Event::delete(&self.keys, vec![Hash::from_str(event_id)?], None)?;
        self.send_event(event)
    }

    pub fn handle_notifications<F>(&self, func: F) -> Result<()>
    where
        F: Fn(RelayPoolNotifications) -> Result<()>,
    {
        let notifications = self.notifications();

        loop {
            select! {
                recv(notifications) -> result => {
                    if let Ok(notification) = result {
                        func(notification)?;
                    }
                }
            }
        }
    }
}
