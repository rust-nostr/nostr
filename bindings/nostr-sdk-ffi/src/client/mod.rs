// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::nips::nip59::UnwrappedGift;
use nostr_ffi::signer::{NostrSigner, NostrSignerFFI2Rust, NostrSignerRust2FFI};
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
pub mod zapper;

pub use self::builder::ClientBuilder;
pub use self::options::{EventSource, Options};
use self::zapper::{ZapDetails, ZapEntity};
use crate::database::events::Events;
use crate::error::Result;
use crate::pool::result::{Output, ReconciliationOutput, SendEventOutput, SubscribeOutput};
use crate::pool::RelayPool;
use crate::relay::options::{SubscribeAutoCloseOptions, SyncOptions};
use crate::relay::RelayFiltering;
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
    #[uniffi::constructor(default(signer = None))]
    pub fn new(signer: Option<Arc<dyn NostrSigner>>) -> Self {
        Self::with_opts(signer, Arc::new(Options::new()))
    }

    #[uniffi::constructor]
    pub fn with_opts(signer: Option<Arc<dyn NostrSigner>>, opts: Arc<Options>) -> Self {
        Self {
            inner: match signer {
                Some(signer) => ClientSdk::with_opts(
                    NostrSignerFFI2Rust::new(signer),
                    opts.as_ref().deref().clone(),
                ),
                None => nostr_sdk::Client::builder()
                    .opts(opts.as_ref().deref().clone())
                    .build(),
            },
        }
    }

    /// Update default difficulty for new `Event`
    #[inline]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    /// Update minimum POW difficulty for received events
    ///
    /// Events with a POW lower than the current value will be ignored to prevent resources exhaustion.
    #[inline]
    pub fn update_min_pow_difficulty(&self, difficulty: u8) {
        self.inner.update_min_pow_difficulty(difficulty);
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn automatic_authentication(&self, enable: bool) {
        self.inner.automatic_authentication(enable);
    }

    pub async fn signer(&self) -> Result<Arc<dyn NostrSigner>> {
        let signer = self.inner.signer().await?;
        let intermediate = NostrSignerRust2FFI::new(signer);
        Ok(Arc::new(intermediate) as Arc<dyn NostrSigner>)
    }

    /// Get relay pool
    pub fn pool(&self) -> Arc<RelayPool> {
        Arc::new(self.inner.pool().into())
    }

    pub fn database(&self) -> NostrDatabase {
        self.inner.database().clone().into()
    }

    /// Get filtering
    pub fn filtering(&self) -> RelayFiltering {
        self.inner.filtering().clone().into()
    }

    pub async fn shutdown(&self) -> Result<()> {
        Ok(self.inner.shutdown().await?)
    }

    /// Get relays with `READ` or `WRITE` flags
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
    /// Relays added with this method will have both `READ` and `WRITE` flags enabled
    ///
    /// If the relay already exists, the flags will be updated and `false` returned.
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    ///
    /// This method use previously set or default `Options` to configure the `Relay` (ex. set proxy, set min POW, set relay limits, ...).
    /// To use custom `RelayOptions` use `add_relay` method on `RelayPool`.
    ///
    /// Connection is **NOT** automatically started with relay, remember to call `connect` method!
    pub async fn add_relay(&self, url: String) -> Result<bool> {
        Ok(self.inner.add_relay(url).await?)
    }

    /// Add discovery relay
    ///
    /// If relay already exists, this method automatically add the `DISCOVERY` flag to it and return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    pub async fn add_discovery_relay(&self, url: String) -> Result<bool> {
        Ok(self.inner.add_discovery_relay(url).await?)
    }

    /// Add read relay
    ///
    /// If relay already exists, this method add the `READ` flag to it and return `false`.
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    pub async fn add_read_relay(&self, url: String) -> Result<bool> {
        Ok(self.inner.add_read_relay(url).await?)
    }

    /// Add write relay
    ///
    /// If relay already exists, this method add the `WRITE` flag to it and return `false`.
    pub async fn add_write_relay(&self, url: String) -> Result<bool> {
        Ok(self.inner.add_write_relay(url).await?)
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has `INBOX` or `OUTBOX` flags, it will not be removed from the pool and its
    /// flags will be updated (remove `READ`, `WRITE` and `DISCOVERY` flags).
    pub async fn remove_relay(&self, url: &str) -> Result<()> {
        Ok(self.inner.remove_relay(url).await?)
    }

    /// Force remove and disconnect relay
    ///
    /// Note: this method will remove the relay, also if it's in use for the gossip model or other service!
    pub async fn force_remove_relay(&self, url: &str) -> Result<()> {
        Ok(self.inner.force_remove_relay(url).await?)
    }

    /// Connect to a previously added relay
    pub async fn connect_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.connect_relay(url).await?)
    }

    pub async fn disconnect_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.disconnect_relay(url).await?)
    }

    /// Connect to all added relays
    pub async fn connect(&self) {
        self.inner.connect().await
    }

    /// Connect to all added relays
    ///
    /// Try to connect to the relays and wait for them to be connected at most for the specified `timeout`.
    /// The code continues if the `timeout` is reached or if all relays connect.
    #[inline]
    pub async fn connect_with_timeout(&self, timeout: Duration) {
        self.inner.connect_with_timeout(timeout).await
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
    /// If `gossip` is enabled (see `Options]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[uniffi::method(default(opts = None))]
    pub async fn subscribe(
        &self,
        filters: Vec<Arc<Filter>>,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> Result<SubscribeOutput> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .subscribe(filters, opts.map(|o| **o))
            .await?
            .into())
    }

    /// Subscribe to filters with custom subscription ID
    ///
    /// If `gossip` is enabled (see `Options]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[uniffi::method(default(opts = None))]
    pub async fn subscribe_with_id(
        &self,
        id: String,
        filters: Vec<Arc<Filter>>,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> Result<Output> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .subscribe_with_id(SubscriptionId::new(id), filters, opts.map(|o| **o))
            .await?
            .into())
    }

    /// Subscribe to filters to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[uniffi::method(default(opts = None))]
    pub async fn subscribe_to(
        &self,
        urls: Vec<String>,
        filters: Vec<Arc<Filter>>,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> Result<SubscribeOutput> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .subscribe_to(urls, filters, opts.map(|o| **o))
            .await?
            .into())
    }

    /// Subscribe to filters with custom subscription ID to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[uniffi::method(default(opts = None))]
    pub async fn subscribe_with_id_to(
        &self,
        urls: Vec<String>,
        id: String,
        filters: Vec<Arc<Filter>>,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> Result<Output> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .subscribe_with_id_to(urls, SubscriptionId::new(id), filters, opts.map(|o| **o))
            .await?
            .into())
    }

    pub async fn unsubscribe(&self, subscription_id: String) {
        self.inner
            .unsubscribe(SubscriptionId::new(subscription_id))
            .await
    }

    pub async fn unsubscribe_all(&self) {
        self.inner.unsubscribe_all().await
    }

    /// Sync events with relays (negentropy reconciliation)
    ///
    /// If `gossip` is enabled (see `Options`) the events will be reconciled also with
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// <https://github.com/hoytech/negentropy>
    pub async fn sync(
        &self,
        filter: Arc<Filter>,
        opts: &SyncOptions,
    ) -> Result<ReconciliationOutput> {
        Ok(self
            .inner
            .sync(filter.as_ref().deref().clone(), opts.deref())
            .await?
            .into())
    }

    /// Fetch events from relays
    ///
    /// If `gossip` is enabled (see `Options`) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    pub async fn fetch_events(
        &self,
        filters: Vec<Arc<Filter>>,
        timeout: Option<Duration>,
    ) -> Result<Events> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self.inner.fetch_events(filters, timeout).await?.into())
    }

    /// Fetch events from specific relays
    pub async fn fetch_events_from(
        &self,
        urls: Vec<String>,
        filters: Vec<Arc<Filter>>,
        timeout: Option<Duration>,
    ) -> Result<Events> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .fetch_events_from(urls, filters, timeout)
            .await?
            .into())
    }

    pub async fn send_msg_to(&self, urls: Vec<String>, msg: Arc<ClientMessage>) -> Result<Output> {
        Ok(self
            .inner
            .send_msg_to(urls, msg.as_ref().deref().clone())
            .await?
            .into())
    }

    /// Send event
    ///
    /// Send event to all relays with `WRITE` flag.
    /// If `gossip` is enabled (see `Options`) the event will be sent also to NIP65 relays (automatically discovered).
    pub async fn send_event(&self, event: Arc<Event>) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event(event.as_ref().deref().clone())
            .await?
            .into())
    }

    /// Send event to specific relays.
    pub async fn send_event_to(
        &self,
        urls: Vec<String>,
        event: Arc<Event>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event_to(urls, event.as_ref().deref().clone())
            .await?
            .into())
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

    /// Take an `EventBuilder`, sign it by using the `NostrSigner` and broadcast to relays (check `send_event` method for more details)
    ///
    /// Rise an error if the `NostrSigner` is not set.
    pub async fn send_event_builder(&self, builder: Arc<EventBuilder>) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event_builder(builder.as_ref().deref().clone())
            .await?
            .into())
    }

    /// Take an `EventBuilder`, sign it by using the `NostrSigner` and broadcast to specific relays.
    ///
    /// Rise an error if the `NostrSigner` is not set.
    pub async fn send_event_builder_to(
        &self,
        urls: Vec<String>,
        builder: Arc<EventBuilder>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event_builder_to(urls, builder.as_ref().deref().clone())
            .await?
            .into())
    }

    /// Fetch the newest public key metadata from database and connected relays.
    ///
    /// If you only want to consult cached data,
    /// consider `client.database().profile(PUBKEY)`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[uniffi::method(default(timeout = None))]
    pub async fn fetch_metadata(
        &self,
        public_key: &PublicKey,
        timeout: Option<Duration>,
    ) -> Result<Arc<Metadata>> {
        Ok(Arc::new(
            self.inner
                .fetch_metadata(**public_key, timeout)
                .await?
                .into(),
        ))
    }

    pub async fn set_metadata(&self, metadata: Arc<Metadata>) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .set_metadata(metadata.as_ref().deref())
            .await?
            .into())
    }

    /// Send private direct message to all relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[uniffi::method(default(reply_to = None))]
    pub async fn send_private_msg(
        &self,
        receiver: &PublicKey,
        message: String,
        reply_to: Option<Arc<EventId>>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_private_msg(**receiver, message, reply_to.map(|t| **t))
            .await?
            .into())
    }

    /// Send private direct message to specific relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[uniffi::method(default(reply_to = None))]
    pub async fn send_private_msg_to(
        &self,
        urls: Vec<String>,
        receiver: &PublicKey,
        message: String,
        reply_to: Option<Arc<EventId>>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_private_msg_to(urls, **receiver, message, reply_to.map(|t| **t))
            .await?
            .into())
    }

    /// Repost
    pub async fn repost(
        &self,
        event: Arc<Event>,
        relay_url: Option<String>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .repost(event.as_ref().deref(), relay_url.map(UncheckedUrl::from))
            .await?
            .into())
    }

    /// Like event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn like(&self, event: Arc<Event>) -> Result<SendEventOutput> {
        Ok(self.inner.like(event.as_ref().deref()).await?.into())
    }

    /// Disike event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn dislike(&self, event: Arc<Event>) -> Result<SendEventOutput> {
        Ok(self.inner.dislike(event.as_ref().deref()).await?.into())
    }

    /// React to an [`Event`]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn reaction(&self, event: Arc<Event>, reaction: String) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .reaction(event.as_ref().deref(), reaction)
            .await?
            .into())
    }

    /// Send a Zap!
    pub async fn zap(
        &self,
        to: &ZapEntity,
        satoshi: u64,
        details: Option<Arc<ZapDetails>>,
    ) -> Result<()> {
        Ok(self
            .inner
            .zap(**to, satoshi, details.map(|d| d.as_ref().deref().clone()))
            .await?)
    }

    /// Construct Gift Wrap and send to relays
    ///
    /// Check `send_event` method to know how sending events works.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    pub async fn gift_wrap(
        &self,
        receiver: &PublicKey,
        rumor: Arc<EventBuilder>,
        expiration: Option<Arc<Timestamp>>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .gift_wrap(
                receiver.deref(),
                rumor.as_ref().deref().clone(),
                expiration.map(|t| **t),
            )
            .await?
            .into())
    }

    /// Construct Gift Wrap and send to specific relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    pub async fn gift_wrap_to(
        &self,
        urls: Vec<String>,
        receiver: &PublicKey,
        rumor: Arc<EventBuilder>,
        expiration: Option<Arc<Timestamp>>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .gift_wrap_to(
                urls,
                receiver.deref(),
                rumor.as_ref().deref().clone(),
                expiration.map(|t| **t),
            )
            .await?
            .into())
    }

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    pub async fn unwrap_gift_wrap(&self, gift_wrap: &Event) -> Result<UnwrappedGift> {
        Ok(self.inner.unwrap_gift_wrap(gift_wrap.deref()).await?.into())
    }

    pub async fn file_metadata(
        &self,
        description: String,
        metadata: Arc<FileMetadata>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .file_metadata(description, metadata.as_ref().deref().clone())
            .await?
            .into())
    }

    /// Handle notifications
    pub async fn handle_notifications(&self, handler: Arc<dyn HandleNotification>) -> Result<()> {
        Ok(self
            .inner
            .handle_notifications(|notification| async {
                match notification {
                    RelayPoolNotificationSdk::Message { relay_url, message } => {
                        handler
                            .handle_msg(relay_url.to_string(), Arc::new(message.into()))
                            .await;
                    }
                    RelayPoolNotificationSdk::Event {
                        relay_url,
                        subscription_id,
                        event,
                    } => {
                        handler
                            .handle(
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
            .await?)
    }
}
