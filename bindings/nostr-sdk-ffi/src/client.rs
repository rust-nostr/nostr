// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt::Debug;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use nostr_ffi::{Event, Filter, Keys};
use nostr_sdk::client::blocking::Client as ClientSdk;
use nostr_sdk::nostr::Filter as FilterSdk;
use nostr_sdk::relay::RelayPoolNotification as RelayPoolNotificationSdk;

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

    pub fn connect_relay(&self, url: String) -> Result<()> {
        Ok(self.client.connect_relay(url)?)
    }

    pub fn connect(&self) {
        self.client.connect()
    }

    pub fn subscribe(&self, filters: Vec<Arc<Filter>>) {
        let mut new_filters: Vec<FilterSdk> = Vec::with_capacity(filters.len());
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
                if let RelayPoolNotificationSdk::Event(url, event) = notification {
                    handler.handle(url.to_string(), Arc::new(event.into()));
                }

                Ok(false)
            })?)
        });
    }
}

pub trait HandleNotification: Send + Sync + Debug {
    fn handle(&self, relay_url: String, event: Arc<Event>);
}
