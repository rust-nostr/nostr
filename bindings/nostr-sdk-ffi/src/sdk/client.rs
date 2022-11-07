// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use nostr_sdk::client::Client as ClientSdk;
use nostr_sdk::relay::RelayPoolNotifications as RelayPoolNotificationsSdk;
use parking_lot::Mutex;

use crate::base::event::{Contact, Event};
use crate::base::key::Keys;
use crate::base::subscription::SubscriptionFilter;

pub struct Client {
    client: Mutex<ClientSdk>,
}

impl Client {
    pub fn new(keys: Arc<Keys>, contacts: Option<Vec<Arc<Contact>>>) -> Self {
        let contacts = match contacts {
            Some(contacts) => {
                let mut c: Vec<nostr_sdk_base::Contact> = Vec::with_capacity(contacts.len());

                for i in contacts.iter() {
                    c.push(i.as_ref().deref().clone());
                }

                Some(c)
            }
            None => None,
        };

        Self {
            client: Mutex::new(ClientSdk::new(keys.as_ref().deref(), contacts)),
        }
    }

    pub fn add_contact(&self, contact: Arc<Contact>) {
        self.client
            .lock()
            .add_contact(contact.as_ref().deref().clone());
    }

    pub fn remove_contact(&self, contact: Arc<Contact>) {
        self.client.lock().remove_contact(contact.as_ref().deref());
    }

    pub fn add_relay(&self, url: String, proxy: Option<String>) -> Result<()> {
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse()?),
            None => None,
        };

        self.client.lock().add_relay(&url, proxy)
    }

    pub fn connect_relay(&self, url: String) -> Result<()> {
        self.client.lock().connect_relay(&url)
    }

    pub fn connect_all(&self) -> Result<()> {
        self.client.lock().connect_all()
    }

    pub fn subscribe(&self, filters: Vec<Arc<SubscriptionFilter>>) -> Result<()> {
        let mut new_filters: Vec<nostr_sdk_base::SubscriptionFilter> =
            Vec::with_capacity(filters.len());
        for filter in filters.into_iter() {
            new_filters.push(filter.as_ref().deref().clone());
        }

        self.client.lock().subscribe(new_filters)
    }

    pub fn send_event(&self, event: Arc<Event>) -> Result<()> {
        self.client
            .lock()
            .send_event(event.as_ref().deref().clone())
    }

    pub fn handle_notifications(self: Arc<Self>, handler: Box<dyn HandleNotification>) {
        nostr_sdk_common::thread::spawn("client", move || {
            log::debug!("Client Thread Started");
            self.client.lock().handle_notifications(|notification| {
                match notification {
                    RelayPoolNotificationsSdk::ReceivedEvent(event) => {
                        handler.handle(Arc::new(event.into()));
                    }
                    RelayPoolNotificationsSdk::RelayDisconnected(url) => {
                        log::debug!("Relay {} disconnected", url);
                    }
                }

                Ok(())
            })
        });
    }
}

pub trait HandleNotification: Send + Sync {
    fn handle(&self, event: Arc<Event>);
}
