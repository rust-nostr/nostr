// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bitcoin_hashes::sha256::Hash;
use crossbeam_channel::{select, Receiver};
use nostr_sdk_base::{Contact, Event, Keys, SubscriptionFilter};
use tokio::sync::Mutex;

use crate::relay::{RelayPool, RelayPoolNotifications};
#[cfg(feature = "blocking")]
use crate::RUNTIME;

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

    pub async fn connect_relay(&self, url: &str) {
        let mut pool = self.pool.lock().await;
        pool.connect_relay(url).await;
    }

    pub async fn connect_all(&self) {
        let mut pool = self.pool.lock().await;
        pool.connect_all().await;
    }

    pub async fn subscribe(&self, filters: Vec<SubscriptionFilter>) {
        let mut pool = self.pool.lock().await;
        pool.start_sub(filters).await;
    }

    pub async fn send_event(&self, event: Event) {
        let pool = self.pool.lock().await;
        pool.send_event(event).await;
    }

    pub async fn delete_event(&self, event_id: &str) -> Result<()> {
        let event = Event::delete(&self.keys, vec![Hash::from_str(event_id)?], None)?;
        self.send_event(event).await;
        Ok(())
    }

    pub async fn keep_alive<F>(&self, func: F) -> Result<()>
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
                default(Duration::from_secs(60)) => self.connect_all().await
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

    pub fn connect_relay(&self, url: &str) {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.connect_relay(url).await;
        });
    }

    pub fn connect_all(&self) {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.connect_all().await;
        });
    }

    pub fn subscribe(&self, filters: Vec<SubscriptionFilter>) {
        RUNTIME.block_on(async {
            let mut pool = self.pool.lock().await;
            pool.start_sub(filters).await;
        });
    }

    pub fn send_event(&self, event: Event) {
        RUNTIME.block_on(async {
            let pool = self.pool.lock().await;
            pool.send_event(event).await;
        });
    }

    pub fn delete_event(&self, event_id: &str) -> Result<()> {
        let event = Event::delete(&self.keys, vec![Hash::from_str(event_id)?], None)?;
        self.send_event(event);
        Ok(())
    }

    pub fn keep_alive<F>(&self, func: F) -> Result<()>
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
                default(Duration::from_secs(60)) => self.connect_all()
            }
        }
    }
}
