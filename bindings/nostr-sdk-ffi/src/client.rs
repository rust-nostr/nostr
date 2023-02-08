// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::{Event, Keys, SubscriptionFilter};
use nostr_sdk::client::blocking::Client as ClientSdk;
use nostr_sdk::relay::pool::RelayPoolNotification as RelayPoolNotificationSdk;

use crate::error::Result;

pub struct Client {
    client: ClientSdk,
}

impl Client {
    pub fn new(keys: Arc<Keys>) -> Self {
        Self {
            client: ClientSdk::new(keys.as_ref().deref()),
        }
    }

    pub fn add_relay(&self, url: String, proxy: Option<String>) -> Result<()> {
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse()?),
            None => None,
        };

        Ok(self.client.add_relay(url, proxy)?)
    }

    pub fn connect_relay(&self, url: String, wait_for_connection: bool) -> Result<()> {
        Ok(self.client.connect_relay(url, wait_for_connection)?)
    }

    pub fn connect(&self) {
        self.client.connect()
    }

    pub fn subscribe(&self, filters: Vec<Arc<SubscriptionFilter>>) {
        let mut new_filters: Vec<nostr::SubscriptionFilter> = Vec::with_capacity(filters.len());
        for filter in filters.into_iter() {
            new_filters.push(filter.as_ref().deref().clone());
        }
        self.client.subscribe(new_filters);
    }

    pub fn send_event(&self, event: Arc<Event>) -> Result<()> {
        self.client.send_event(event.as_ref().deref().clone())?;
        Ok(())
    }

    pub fn handle_notifications(self: Arc<Self>, handler: Box<dyn HandleNotification>) {
        crate::thread::spawn("client", move || {
            log::debug!("Client Thread Started");
            Ok(self.client.handle_notifications(|notification| {
                if let RelayPoolNotificationSdk::Event(_url, event) = notification {
                    handler.handle(Arc::new(event.into()));
                }

                Ok(())
            })?)
        });
    }
}

pub trait HandleNotification: Send + Sync {
    fn handle(&self, event: Arc<Event>);
}
