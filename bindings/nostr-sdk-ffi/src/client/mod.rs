// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use async_utility::thread;
use nostr_ffi::{
    ClientMessage, Event, EventBuilder, EventId, FileMetadata, Filter, Metadata, PublicKey,
    Timestamp,
};
use nostr_sdk::client::Client as ClientSdk;
use nostr_sdk::pool::RelayPoolNotification as RelayPoolNotificationSdk;
use nostr_sdk::{SubscriptionId, UncheckedUrl};
use uniffi::Object;

mod builder;
mod options;
pub mod signer;
pub mod zapper;

pub use self::builder::ClientBuilder;
pub use self::options::Options;
pub use self::signer::NostrSigner;
use self::zapper::{ZapDetails, ZapEntity};
use crate::abortable::AbortHandle;
use crate::error::Result;
use crate::relay::options::{NegentropyOptions, SubscribeAutoCloseOptions};
use crate::relay::RelayOptions;
use crate::{HandleNotification, NostrDatabase, Relay};

#[derive(Object)]
pub struct Client {
    inner: ClientSdk,
}

impl From<ClientSdk> for Client {
    fn from(inner: ClientSdk) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl Client {
    #[uniffi::constructor]
    pub fn new(signer: Option<Arc<NostrSigner>>) -> Self {
        Self::with_opts(signer, Arc::new(Options::new()))
    }

    #[uniffi::constructor]
    pub fn with_opts(signer: Option<Arc<NostrSigner>>, opts: Arc<Options>) -> Self {
        Self {
            inner: match signer {
                Some(signer) => ClientSdk::with_opts(
                    signer.as_ref().deref().clone(),
                    opts.as_ref().deref().clone(),
                ),
                None => nostr_sdk::ClientBuilder::new()
                    .opts(opts.as_ref().deref().clone())
                    .build(),
            },
        }
    }

    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    pub async fn signer(&self) -> Result<NostrSigner> {
        Ok(self.inner.signer().await?.into())
    }

    pub fn database(&self) -> Arc<NostrDatabase> {
        Arc::new(self.inner.database().into())
    }

    pub async fn start(&self) {
        self.inner.start().await
    }

    pub async fn stop(&self) -> Result<()> {
        Ok(self.inner.stop().await?)
    }

    pub async fn shutdown(&self) -> Result<()> {
        Ok(self.inner.clone().shutdown().await?)
    }

    pub async fn relays(&self) -> HashMap<String, Arc<Relay>> {
        self.inner
            .relays()
            .await
            .into_iter()
            .map(|(u, r)| (u.to_string(), Arc::new(r.into())))
            .collect()
    }

    pub async fn relay(&self, url: String) -> Result<Arc<Relay>> {
        Ok(Arc::new(self.inner.relay(url).await?.into()))
    }

    /// Add new relay
    ///
    /// Return `false` if the relay already exists.
    ///
    /// This method use perviously set or default `Options` to configure the `Relay` (ex. set proxy, set min POW, set relay limits, ...).
    /// To use custom `RelayOptions`, check `add_relay_with_opts` method.
    ///
    /// Connection is **NOT** automatically started with relay, remember to call `connect` method!
    pub async fn add_relay(&self, url: String) -> Result<bool> {
        Ok(self.inner.add_relay(url).await?)
    }

    /// Add new relay with custom `RelayOptions`
    ///
    /// Return `false` if the relay already exists.
    ///
    /// Connection is **NOT** automatically started with relay, remember to call `connect` method!
    pub async fn add_relay_with_opts(&self, url: String, opts: &RelayOptions) -> Result<bool> {
        Ok(self
            .inner
            .add_relay_with_opts(url, opts.deref().clone())
            .await?)
    }

    /// Add multiple relays
    ///
    /// Connection is **NOT** automatically started with relays, remember to call `connect` method!
    pub async fn add_relays(&self, relays: Vec<String>) -> Result<()> {
        Ok(self.inner.add_relays(relays).await?)
    }

    pub async fn remove_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.remove_relay(url).await?)
    }

    pub async fn connect_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.connect_relay(url).await?)
    }

    pub async fn disconnect_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.disconnect_relay(url).await?)
    }

    pub async fn connect(&self) {
        self.inner.connect().await
    }

    pub async fn disconnect(&self) -> Result<()> {
        Ok(self.inner.disconnect().await?)
    }

    pub async fn subscriptions(&self) -> HashMap<String, Vec<Arc<Filter>>> {
        self.inner
            .subscriptions()
            .await
            .into_iter()
            .map(|(id, filters)| {
                (
                    id.to_string(),
                    filters.into_iter().map(|f| Arc::new(f.into())).collect(),
                )
            })
            .collect()
    }

    pub async fn subscription(&self, id: String) -> Option<Vec<Arc<Filter>>> {
        self.inner
            .subscription(&SubscriptionId::new(id))
            .await
            .map(|filters| filters.into_iter().map(|f| Arc::new(f.into())).collect())
    }

    /// Subscribe to filters
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    pub async fn subscribe(
        &self,
        filters: Vec<Arc<Filter>>,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> String {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();

        self.inner
            .subscribe(filters, opts.map(|o| **o))
            .await
            .to_string()
    }

    /// Subscribe to filters with custom subscription ID
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    pub async fn subscribe_with_id(
        &self,
        id: String,
        filters: Vec<Arc<Filter>>,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();

        self.inner
            .subscribe_with_id(SubscriptionId::new(id), filters, opts.map(|o| **o))
            .await
    }

    pub async fn unsubscribe(&self, subscription_id: String) {
        self.inner
            .unsubscribe(SubscriptionId::new(subscription_id))
            .await
    }

    pub async fn unsubscribe_all(&self) {
        self.inner.unsubscribe_all().await
    }

    pub async fn get_events_of(
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
            .get_events_of(filters, timeout)
            .await?
            .into_iter()
            .map(|e| Arc::new(e.into()))
            .collect())
    }

    /// Get events of filters from specific relays
    ///
    /// Get events both from **local database** and **relays**
    ///
    /// If no relay is specified, will be queried only the database.
    pub async fn get_events_from(
        &self,
        urls: Vec<String>,
        filters: Vec<Arc<Filter>>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Arc<Event>>> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .get_events_from(urls, filters, timeout)
            .await?
            .into_iter()
            .map(|e| Arc::new(e.into()))
            .collect())
    }

    pub async fn send_msg(&self, msg: Arc<ClientMessage>) -> Result<()> {
        Ok(self.inner.send_msg(msg.as_ref().deref().clone()).await?)
    }

    pub async fn send_msg_to(&self, urls: Vec<String>, msg: Arc<ClientMessage>) -> Result<()> {
        Ok(self
            .inner
            .send_msg_to(urls, msg.as_ref().deref().clone())
            .await?)
    }

    pub async fn send_event(&self, event: Arc<Event>) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .send_event(event.as_ref().deref().clone())
                .await?
                .into(),
        ))
    }

    pub async fn send_event_to(
        &self,
        urls: Vec<String>,
        event: Arc<Event>,
    ) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .send_event_to(urls, event.as_ref().deref().clone())
                .await?
                .into(),
        ))
    }

    /// Signs the `EventBuilder` into an `Event` using the `NostrSigner`
    pub async fn sign_event_builder(&self, builder: Arc<EventBuilder>) -> Result<Arc<Event>> {
        Ok(Arc::new(
            self.inner
                .sign_event_builder(builder.as_ref().deref().clone())
                .await?
                .into(),
        ))
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to all relays.
    ///
    /// Rise an error if the [`NostrSigner`] is not set.
    pub async fn send_event_builder(&self, builder: Arc<EventBuilder>) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .send_event_builder(builder.as_ref().deref().clone())
                .await?
                .into(),
        ))
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to specific relays.
    ///
    /// Rise an error if the [`NostrSigner`] is not set.
    pub async fn send_event_builder_to(
        &self,
        urls: Vec<String>,
        builder: Arc<EventBuilder>,
    ) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .send_event_builder_to(urls, builder.as_ref().deref().clone())
                .await?
                .into(),
        ))
    }

    pub async fn set_metadata(&self, metadata: Arc<Metadata>) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .set_metadata(metadata.as_ref().deref())
                .await?
                .into(),
        ))
    }

    pub async fn send_direct_msg(
        &self,
        receiver: &PublicKey,
        msg: String,
        reply: Option<Arc<EventId>>,
    ) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .send_direct_msg(**receiver, msg, reply.map(|r| **r))
                .await?
                .into(),
        ))
    }

    /// Repost
    pub async fn repost(
        &self,
        event: Arc<Event>,
        relay_url: Option<String>,
    ) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .repost(event.as_ref().deref(), relay_url.map(UncheckedUrl::from))
                .await?
                .into(),
        ))
    }

    /// Like event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn like(&self, event: Arc<Event>) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner.like(event.as_ref().deref()).await?.into(),
        ))
    }

    /// Disike event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn dislike(&self, event: Arc<Event>) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner.dislike(event.as_ref().deref()).await?.into(),
        ))
    }

    /// React to an [`Event`]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn reaction(&self, event: Arc<Event>, reaction: String) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .reaction(event.as_ref().deref(), reaction)
                .await?
                .into(),
        ))
    }

    /// Send a Zap!
    ///
    /// This method automatically create a split zap to support Rust Nostr development.
    pub async fn zap(
        &self,
        to: Arc<ZapEntity>,
        satoshi: u64,
        details: Option<Arc<ZapDetails>>,
    ) -> Result<()> {
        Ok(self
            .inner
            .zap(**to, satoshi, details.map(|d| d.as_ref().deref().clone()))
            .await?)
    }

    /// Gift Wrap
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    pub async fn gift_wrap(
        &self,
        receiver: &PublicKey,
        rumor: Arc<EventBuilder>,
        expiration: Option<Arc<Timestamp>>,
    ) -> Result<()> {
        Ok(self
            .inner
            .gift_wrap(
                **receiver,
                rumor.as_ref().deref().clone(),
                expiration.map(|t| **t),
            )
            .await?)
    }

    /// Send GiftWrapper Sealed Direct message
    pub async fn send_sealed_msg(
        &self,
        receiver: &PublicKey,
        message: String,
        expiration: Option<Arc<Timestamp>>,
    ) -> Result<()> {
        Ok(self
            .inner
            .send_sealed_msg(**receiver, message, expiration.map(|t| **t))
            .await?)
    }

    pub async fn file_metadata(
        &self,
        description: String,
        metadata: Arc<FileMetadata>,
    ) -> Result<Arc<EventId>> {
        Ok(Arc::new(
            self.inner
                .file_metadata(description, metadata.as_ref().deref().clone())
                .await?
                .into(),
        ))
    }

    pub async fn reconcile(&self, filter: Arc<Filter>, opts: Arc<NegentropyOptions>) -> Result<()> {
        Ok(self
            .inner
            .reconcile(filter.as_ref().deref().clone(), **opts)
            .await?)
    }

    /// Handle notifications
    ///
    /// **This method spawn a thread**, so ensure to keep up the app after calling this (if needed).
    pub fn handle_notifications(
        &self,
        handler: Arc<dyn HandleNotification>,
    ) -> Result<Arc<AbortHandle>> {
        let client = self.inner.clone();
        let h = handler.clone();
        let handle = thread::abortable(async move {
            let _ = client
                .handle_notifications(|notification| async {
                    match notification {
                        RelayPoolNotificationSdk::Message { relay_url, message } => {
                            h.handle_msg(relay_url.to_string(), Arc::new(message.into()))
                                .await;
                        }
                        RelayPoolNotificationSdk::Event {
                            relay_url,
                            subscription_id,
                            event,
                        } => {
                            h.handle(
                                relay_url.to_string(),
                                subscription_id.to_string(),
                                Arc::new((*event).into()),
                            )
                            .await;
                        }
                        _ => (),
                    }
                    Ok(false)
                })
                .await;
        })?;
        Ok(Arc::new(handle.into()))
    }
}
