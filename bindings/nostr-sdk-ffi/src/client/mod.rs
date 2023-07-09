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

mod options;

pub use self::options::Options;
use crate::error::Result;

pub struct Client {
    inner: ClientSdk,
}

impl Client {
    pub fn new(keys: Arc<Keys>) -> Self {
        Self {
            inner: ClientSdk::new(keys.as_ref().deref()),
        }
    }

    pub fn with_opts(keys: Arc<Keys>, opts: Arc<Options>) -> Self {
        Self {
            inner: ClientSdk::with_opts(keys.as_ref().deref(), opts.as_ref().deref().clone()),
        }
    }

    // TODO: add with_remote_signer

    // TODO: add with_remote_signer_and_opts

    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    pub fn keys(&self) -> Arc<Keys> {
        Arc::new(self.inner.keys().into())
    }

    // TODO: add nostr_connect_uri

    // TODO: add remote_signer

    pub fn start(&self) {
        self.inner.start();
    }

    pub fn stop(&self) -> Result<()> {
        Ok(self.inner.stop()?)
    }

    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    pub fn shutdown(&self) -> Result<()> {
        Ok(self.inner.clone().shutdown()?)
    }

    pub fn clear_already_seen_events(&self) {
        self.inner.clear_already_seen_events()
    }

    // TODO: add relays

    // TODO: add relay

    pub fn add_relay(&self, url: String, proxy: Option<String>) -> Result<()> {
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse()?),
            None => None,
        };

        Ok(self.inner.add_relay(url, proxy)?)
    }

    // TODO: add add_relay_with_opts

    pub fn remove_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.remove_relay(url)?)
    }

    // TODO: add add_relays

    pub fn connect_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.connect_relay(url)?)
    }

    pub fn disconnect_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.disconnect_relay(url)?)
    }

    pub fn connect(&self) {
        self.inner.connect()
    }

    pub fn disconnect(&self) -> Result<()> {
        Ok(self.inner.disconnect()?)
    }

    pub fn subscribe(&self, filters: Vec<Arc<Filter>>) {
        let mut new_filters: Vec<FilterSdk> = Vec::with_capacity(filters.len());
        for filter in filters.into_iter() {
            new_filters.push(filter.as_ref().deref().clone());
        }
        self.inner.subscribe(new_filters);
    }

    pub fn send_event(&self, event: Arc<Event>) -> Result<()> {
        self.inner.send_event(event.as_ref().deref().clone())?;
        Ok(())
    }

    pub fn handle_notifications(self: Arc<Self>, handler: Box<dyn HandleNotification>) {
        crate::thread::spawn("client", move || {
            log::debug!("Client Thread Started");
            Ok(self.inner.handle_notifications(|notification| {
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
