// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bitcoin_hashes::sha256::Hash;
use crossbeam_channel::{select, Receiver};
use nostr::{Contact, Event, Keys, SubscriptionFilter};
use tokio::sync::Mutex;

use crate::relay::{RelayPool, RelayPoolNotifications};

pub struct Client {
    pub pool: Arc<Mutex<RelayPool>>,
    keys: Keys,
    contacts: Arc<Mutex<Vec<Contact>>>,
}

impl Client {
    pub fn new(keys: Keys, contacts: Option<Vec<Contact>>) -> Self {
        Self {
            pool: Arc::new(Mutex::new(RelayPool::new())),
            keys,
            contacts: Arc::new(Mutex::new(contacts.unwrap_or_default())),
        }
    }

    pub fn generate_keys() -> Keys {
        Keys::generate_from_os_random()
    }

    pub async fn add_contact(&self, contact: Contact) {
        let mut contacts = self.contacts.lock().await;
        if !contacts.contains(&contact) {
            contacts.push(contact);
        }
    }

    pub async fn remove_contact(&self, contact: Contact) {
        let mut contacts = self.contacts.lock().await;
        if contacts.contains(&contact) {
            contacts.retain(|c| c != &contact);
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

    pub async fn connect_relay(&mut self, url: &str) {
        let mut pool = self.pool.lock().await;
        pool.connect_relay(url).await;
    }

    pub async fn connect_all(&self) {
        let mut pool = self.pool.lock().await;
        pool.connect_all().await;
    }

    pub async fn subscribe(&self, filter: Vec<SubscriptionFilter>) {
        let mut pool = self.pool.lock().await;
        pool.start_sub(filter).await;
    }

    pub async fn send_event(&self, event: Event) {
        let pool = self.pool.lock().await;
        pool.send_event(event).await;
    }

    pub async fn delete_event(&self, event_id: &str) -> Result<()> {
        let event = Event::delete(&self.keys, vec![Hash::from_str(event_id)?], "")?;
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
