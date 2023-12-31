// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::{
    ClientMessage, Event, EventId, FileMetadata, Filter, Metadata, PublicKey, RelayMessage,
};
use nostr_sdk::client::blocking::Client as ClientSdk;
use nostr_sdk::relay::RelayPoolNotification as RelayPoolNotificationSdk;
use nostr_sdk::{NegentropyOptions, Options as OptionsSdk};
use uniffi::Object;

mod builder;
mod options;
pub mod signer;

pub use self::builder::ClientBuilder;
pub use self::options::Options;
pub use self::signer::ClientSigner;
use crate::error::Result;
use crate::{NostrDatabase, Relay};

#[derive(Object)]
pub struct Client {
    inner: ClientSdk,
}

impl From<ClientSdk> for Client {
    fn from(inner: ClientSdk) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Client {
    #[uniffi::constructor]
    pub fn new(signer: Option<ClientSigner>) -> Self {
        Self::with_opts(signer, Arc::new(Options::new()))
    }

    #[uniffi::constructor]
    pub fn with_opts(signer: Option<ClientSigner>, opts: Arc<Options>) -> Self {
        let opts: OptionsSdk = opts.as_ref().deref().clone().shutdown_on_drop(true);
        let mut builder = nostr_sdk::ClientBuilder::new().opts(opts);
        if let Some(signer) = signer {
            let signer: nostr_sdk::ClientSigner = signer.into();
            builder = builder.signer(signer);
        }
        Self {
            inner: builder.build().into(),
        }
    }

    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    pub fn signer(&self) -> Result<ClientSigner> {
        Ok(self.inner.signer()?.into())
    }

    pub fn database(&self) -> Arc<NostrDatabase> {
        Arc::new(self.inner.database().into())
    }

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

    pub fn add_relay(&self, url: String) -> Result<bool> {
        Ok(self.inner.add_relay(url)?)
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
        Ok(self.inner.send_msg(msg.into())?)
    }

    // TODO: add send_msg_with_custom_wait

    pub fn send_msg_to(&self, url: String, msg: ClientMessage) -> Result<()> {
        Ok(self.inner.send_msg_to(url, msg.into())?)
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
        Ok(Arc::new(
            self.inner.set_metadata(metadata.as_ref().deref())?.into(),
        ))
    }

    pub fn send_direct_msg(
        &self,
        receiver: Arc<PublicKey>,
        msg: String,
        reply: Option<Arc<EventId>>,
    ) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .send_direct_msg(**receiver, msg, reply.map(|r| **r))?
                .into(),
        ))
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

    pub fn reconcile(&self, filter: Arc<Filter>) -> Result<()> {
        Ok(self.inner.reconcile(
            filter.as_ref().deref().clone(),
            NegentropyOptions::default(),
        )?)
    }

    pub fn handle_notifications(self: Arc<Self>, handler: Box<dyn HandleNotification>) {
        crate::thread::spawn("client", move || {
            tracing::debug!("Client Thread Started");
            Ok(self.inner.handle_notifications(|notification| {
                match notification {
                    RelayPoolNotificationSdk::Message { relay_url, message } => {
                        handler.handle_msg(relay_url.to_string(), message.into())
                    }
                    RelayPoolNotificationSdk::Event { relay_url, event } => {
                        handler.handle(relay_url.to_string(), Arc::new(event.into()))
                    }
                    _ => (),
                }

                Ok(false)
            })?)
        });
    }
}

#[uniffi::export(callback_interface)]
pub trait HandleNotification: Send + Sync + Debug {
    fn handle_msg(&self, relay_url: String, msg: RelayMessage);
    fn handle(&self, relay_url: String, event: Arc<Event>);
}
