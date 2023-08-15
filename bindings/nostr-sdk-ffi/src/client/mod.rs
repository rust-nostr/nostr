// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::{ClientMessage, Event, EventId, FileMetadata, Filter, Keys, RelayMessage, Metadata};
use nostr_sdk::client::blocking::Client as ClientSdk;
use nostr_sdk::relay::RelayPoolNotification as RelayPoolNotificationSdk;
use nostr_sdk::Options as OptionsSdk;

mod options;

pub use self::options::Options;
use crate::error::Result;
use crate::Relay;

pub struct Client {
    inner: ClientSdk,
}

impl Client {
    pub fn new(keys: Arc<Keys>) -> Self {
        let opts: OptionsSdk = OptionsSdk::new().shutdown_on_drop(true);
        Self {
            inner: ClientSdk::with_opts(keys.as_ref().deref(), opts),
        }
    }

    pub fn with_opts(keys: Arc<Keys>, opts: Arc<Options>) -> Self {
        let opts: OptionsSdk = opts.as_ref().deref().clone().shutdown_on_drop(true);
        Self {
            inner: ClientSdk::with_opts(keys.as_ref().deref(), opts),
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

    pub fn relays(&self) -> HashMap<String, Arc<Relay>> {
        self.inner
            .relays()
            .into_iter()
            .map(|(u, r)| (u.to_string(), Arc::new(r.into())))
            .collect()
    }

    pub fn relay(&self, url: String) -> Result<Arc<Relay>> {
        Ok(Arc::new(self.inner.relay(url)?.into()))
    }

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
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        self.inner.subscribe(filters);
    }

    // TODO: add subscribe_with_custom_wait

    pub fn unsubscribe(&self) {
        self.inner.unsubscribe();
    }

    // TODO: add unsubscribe_with_custom_wait

    pub fn get_events_of(
        &self,
        filters: Vec<Arc<Filter>>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Arc<Event>>> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .get_events_of(filters, timeout)?
            .into_iter()
            .map(|e| Arc::new(e.into()))
            .collect())
    }

    // TODO: add get_events_of_with_opts

    pub fn req_events_of(&self, filters: Vec<Arc<Filter>>, timeout: Option<Duration>) {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        self.inner.req_events_of(filters, timeout);
    }

    // TODO: add req_events_of_with_opts

    pub fn send_msg(&self, msg: ClientMessage) -> Result<()> {
        Ok(self.inner.send_msg(msg.try_into()?)?)
    }

    // TODO: add send_msg_with_custom_wait

    pub fn send_msg_to(&self, url: String, msg: ClientMessage) -> Result<()> {
        Ok(self.inner.send_msg_to(url, msg.try_into()?)?)
    }

    // TODO: add send_msg_to_with_custom_wait

    pub fn send_event(&self, event: Arc<Event>) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .send_event(event.as_ref().deref().clone())?
                .into(),
        ))
    }

    // TODO: add send_event_with_custom_wait

    pub fn send_event_to(&self, url: String, event: Arc<Event>) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .send_event_to(url, event.as_ref().deref().clone())?
                .into(),
        ))
    }

    // TODO: add send_event_to_with_custom_wait

    pub fn set_metadata(&self, metadata: Arc<Metadata>) -> Result<Arc<EventId>> {
        Ok(Arc::new(self.inner.set_metadata(metadata.as_ref().deref().clone())?.into()))
    }

    pub fn file_metadata(
        &self,
        description: String,
        metadata: Arc<FileMetadata>,
    ) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .file_metadata(description, metadata.as_ref().deref().clone())?
                .into(),
        ))
    }

    pub fn handle_notifications(self: Arc<Self>, handler: Box<dyn HandleNotification>) {
        crate::thread::spawn("client", move || {
            tracing::debug!("Client Thread Started");
            Ok(self.inner.handle_notifications(|notification| {
                match notification {
                    RelayPoolNotificationSdk::Message(url, msg) => {
                        handler.handle_msg(url.to_string(), msg.into())
                    }
                    RelayPoolNotificationSdk::Event(url, event) => {
                        handler.handle(url.to_string(), Arc::new(event.into()))
                    }
                    _ => (),
                }

                Ok(false)
            })?)
        });
    }
}

pub trait HandleNotification: Send + Sync + Debug {
    fn handle_msg(&self, relay_url: String, msg: RelayMessage);
    fn handle(&self, relay_url: String, event: Arc<Event>);
}
