// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::nips::nip59::UnwrappedGift;
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
use crate::error::Result;
use crate::pool::result::{Output, SendEventOutput};
use crate::relay::options::{NegentropyOptions, SubscribeAutoCloseOptions};
use crate::relay::{RelayBlacklist, RelayOptions};
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

    pub async fn signer(&self) -> Result<NostrSigner> {
        Ok(self.inner.signer().await?.into())
    }

    pub fn database(&self) -> Arc<NostrDatabase> {
        Arc::new(self.inner.database().into())
    }

    /// Get blacklist
    pub fn blacklist(&self) -> RelayBlacklist {
        self.inner.blacklist().into()
    }

    /// Mute event IDs
    ///
    /// Add event IDs to blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    pub async fn mute_ids(&self, ids: Vec<Arc<EventId>>) {
        self.inner.mute_ids(ids.into_iter().map(|id| **id)).await
    }

    /// Unmute event IDs
    ///
    /// Remove event IDs from blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    pub async fn unmute_ids(&self, ids: &[Arc<EventId>]) {
        self.inner
            .unmute_ids(ids.iter().map(|id| id.as_ref().deref()))
            .await
    }

    /// Mute public keys
    ///
    /// Add public keys to blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    pub async fn mute_public_keys(&self, public_keys: Vec<Arc<PublicKey>>) {
        self.inner
            .mute_public_keys(public_keys.into_iter().map(|p| **p))
            .await
    }

    /// Unmute public keys
    ///
    /// Remove public keys from blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    pub async fn unmute_public_keys(&self, public_keys: &[Arc<PublicKey>]) {
        self.inner
            .unmute_public_keys(public_keys.iter().map(|p| p.as_ref().deref()))
            .await
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
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    ///
    /// This method use previously set or default `Options` to configure the `Relay` (ex. set proxy, set min POW, set relay limits, ...).
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
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
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
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    ///
    /// Connection is **NOT** automatically started with relays, remember to call `connect` method!
    pub async fn add_relays(&self, relays: Vec<String>) -> Result<()> {
        Ok(self.inner.add_relays(relays).await?)
    }

    pub async fn remove_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.remove_relay(url).await?)
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

    /// Subscribe to filters to all connected relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[uniffi::method(default(opts = None))]
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

    /// Subscribe to filters with custom subscription ID to all connected relays
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
    ) {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();

        self.inner
            .subscribe_with_id(SubscriptionId::new(id), filters, opts.map(|o| **o))
            .await
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
    ) -> Result<String> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .subscribe_to(urls, filters, opts.map(|o| **o))
            .await?
            .to_string())
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
    ) -> Result<()> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .subscribe_with_id_to(urls, SubscriptionId::new(id), filters, opts.map(|o| **o))
            .await?)
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

    pub async fn send_msg(&self, msg: Arc<ClientMessage>) -> Result<Output> {
        Ok(self
            .inner
            .send_msg(msg.as_ref().deref().clone())
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

    pub async fn send_event(&self, event: Arc<Event>) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event(event.as_ref().deref().clone())
            .await?
            .into())
    }

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

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to all relays.
    ///
    /// Rise an error if the [`NostrSigner`] is not set.
    pub async fn send_event_builder(&self, builder: Arc<EventBuilder>) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event_builder(builder.as_ref().deref().clone())
            .await?
            .into())
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to specific relays.
    ///
    /// Rise an error if the [`NostrSigner`] is not set.
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

    pub async fn set_metadata(&self, metadata: Arc<Metadata>) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .set_metadata(metadata.as_ref().deref())
            .await?
            .into())
    }

    /// Encrypted direct msg
    ///
    /// <div class="warning"><strong>Unsecure!</strong> Use `send_private_msg` instead!</div>
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/04.md>
    pub async fn send_direct_msg(
        &self,
        receiver: &PublicKey,
        msg: String,
        reply: Option<Arc<EventId>>,
    ) -> Result<SendEventOutput> {
        #[allow(deprecated)]
        Ok(self
            .inner
            .send_direct_msg(**receiver, msg, reply.map(|r| **r))
            .await?
            .into())
    }

    /// Send private direct message
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

    /// Gift Wrap
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
                **receiver,
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

    pub async fn reconcile(
        &self,
        filter: Arc<Filter>,
        opts: Arc<NegentropyOptions>,
    ) -> Result<Output> {
        Ok(self
            .inner
            .reconcile(filter.as_ref().deref().clone(), **opts)
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
