// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_sdk::client::Client as ClientSdk;
use nostr_sdk::pool::RelayPoolNotification as RelayPoolNotificationSdk;
use nostr_sdk::SubscriptionId;
use uniffi::Object;

mod builder;
mod options;
mod output;

pub use self::builder::ClientBuilder;
pub use self::options::Options;
use self::output::{Output, ReconciliationOutput, SendEventOutput, SubscribeOutput};
use crate::database::events::Events;
use crate::database::NostrDatabase;
use crate::error::Result;
use crate::notifications::HandleNotification;
use crate::protocol::event::{Event, EventBuilder, Tag, UnsignedEvent};
use crate::protocol::filter::Filter;
use crate::protocol::key::PublicKey;
use crate::protocol::message::ClientMessage;
use crate::protocol::nips::nip01::Metadata;
use crate::protocol::nips::nip59::UnwrappedGift;
use crate::protocol::signer::NostrSigner;
use crate::relay::options::{SubscribeAutoCloseOptions, SyncOptions};
use crate::relay::{Relay, RelayOptions};

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
        Self {
            inner: match signer {
                Some(signer) => ClientSdk::new(signer.as_ref().deref().clone()),
                None => ClientSdk::default(),
            },
        }
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    pub fn automatic_authentication(&self, enable: bool) {
        self.inner.automatic_authentication(enable);
    }

    pub async fn signer(&self) -> Result<NostrSigner> {
        let signer = self.inner.signer().await?;
        Ok(signer.into())
    }

    pub fn database(&self) -> NostrDatabase {
        self.inner.database().clone().into()
    }

    pub async fn shutdown(&self) {
        self.inner.shutdown().await
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

    pub async fn relay(&self, url: &str) -> Result<Relay> {
        Ok(self.inner.relay(url).await?.into())
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
    ///
    /// Connection is **NOT** automatically started with relay, remember to call `connect` method!
    pub async fn add_relay(&self, url: &str) -> Result<bool> {
        Ok(self.inner.add_relay(url).await?)
    }

    /// Add new relay with custom options
    pub async fn add_relay_with_opts(&self, url: &str, opts: &RelayOptions) -> Result<bool> {
        Ok(self
            .inner
            .pool()
            .add_relay(url, opts.deref().clone())
            .await?)
    }

    /// Add discovery relay
    ///
    /// If relay already exists, this method automatically add the `DISCOVERY` flag to it and return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    pub async fn add_discovery_relay(&self, url: &str) -> Result<bool> {
        Ok(self.inner.add_discovery_relay(url).await?)
    }

    /// Add read relay
    ///
    /// If relay already exists, this method add the `READ` flag to it and return `false`.
    pub async fn add_read_relay(&self, url: &str) -> Result<bool> {
        Ok(self.inner.add_read_relay(url).await?)
    }

    /// Add write relay
    ///
    /// If relay already exists, this method add the `WRITE` flag to it and return `false`.
    pub async fn add_write_relay(&self, url: &str) -> Result<bool> {
        Ok(self.inner.add_write_relay(url).await?)
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has `GOSSIP` flag, it will not be removed from the pool and its
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

    /// Disconnect and remove all relays
    ///
    /// Some relays used by some services could not be disconnected with this method
    /// (like the ones used for gossip).
    /// Use [`Client::force_remove_all_relays`] to remove every relay.
    pub async fn remove_all_relays(&self) {
        self.inner.remove_all_relays().await
    }

    /// Disconnect and force remove all relays
    pub async fn force_remove_all_relays(&self) {
        self.inner.force_remove_all_relays().await
    }

    /// Connect to a previously added relay
    pub async fn connect_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.connect_relay(url).await?)
    }

    pub async fn disconnect_relay(&self, url: String) -> Result<()> {
        Ok(self.inner.disconnect_relay(url).await?)
    }

    /// Connect to all added relays
    ///
    /// Attempts to initiate a connection for every relay currently in
    /// [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`].
    /// A background connection task is spawned for each such relay, which then tries
    /// to establish the connection.
    /// Any relay not in one of these two statuses is skipped.
    ///
    /// For further details, see the documentation of [`Relay::connect`].
    pub async fn connect(&self) {
        self.inner.connect().await
    }

    /// Waits for relays connections
    ///
    /// Wait for relays connections at most for the specified `timeout`.
    /// The code continues when the relays are connected or the `timeout` is reached.
    pub async fn wait_for_connection(&self, timeout: Duration) {
        self.inner.wait_for_connection(timeout).await
    }

    /// Try to establish a connection with the relays.
    ///
    /// Attempts to establish a connection for every relay currently in
    /// [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`]
    /// without spawning the connection task if it fails.
    /// This means that if the connection fails, no automatic retries are scheduled.
    /// Use [`Client::connect`] if you want to immediately spawn a connection task,
    /// regardless of whether the initial connection succeeds.
    ///
    /// For further details, see the documentation of [`Relay::try_connect`].
    pub async fn try_connect(&self, timeout: Duration) -> Output {
        self.inner.try_connect(timeout).await.into()
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) {
        self.inner.disconnect().await
    }

    pub async fn subscriptions(&self) -> HashMap<String, Arc<Filter>> {
        self.inner
            .subscriptions()
            .await
            .into_iter()
            .map(|(id, f)| (id.to_string(), Arc::new(f.into())))
            .collect()
    }

    pub async fn subscription(&self, id: String) -> Option<Arc<Filter>> {
        self.inner
            .subscription(&SubscriptionId::new(id))
            .await
            .map(|f| Arc::new(f.into()))
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
        filter: &Filter,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> Result<SubscribeOutput> {
        Ok(self
            .inner
            .subscribe(filter.deref().clone(), opts.map(|o| **o))
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
        filter: &Filter,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> Result<Output> {
        Ok(self
            .inner
            .subscribe_with_id(
                SubscriptionId::new(id),
                filter.deref().clone(),
                opts.map(|o| **o),
            )
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
        filter: &Filter,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> Result<SubscribeOutput> {
        Ok(self
            .inner
            .subscribe_to(urls, filter.deref().clone(), opts.map(|o| **o))
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
        filter: &Filter,
        opts: Option<Arc<SubscribeAutoCloseOptions>>,
    ) -> Result<Output> {
        Ok(self
            .inner
            .subscribe_with_id_to(
                urls,
                SubscriptionId::new(id),
                filter.deref().clone(),
                opts.map(|o| **o),
            )
            .await?
            .into())
    }

    pub async fn unsubscribe(&self, subscription_id: String) {
        self.inner
            .unsubscribe(&SubscriptionId::new(subscription_id))
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
    pub async fn sync(&self, filter: &Filter, opts: &SyncOptions) -> Result<ReconciliationOutput> {
        Ok(self
            .inner
            .sync(filter.deref().clone(), opts.deref())
            .await?
            .into())
    }

    /// Fetch events from relays
    ///
    /// This is an auto-closing subscription and will be closed automatically on `EOSE`.
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see `Options`) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    pub async fn fetch_events(&self, filter: &Filter, timeout: Duration) -> Result<Events> {
        Ok(self
            .inner
            .fetch_events(filter.deref().clone(), timeout)
            .await?
            .into())
    }

    /// Fetch events from specific relays
    ///
    /// This is an auto-closing subscription and will be closed automatically on `EOSE`.
    pub async fn fetch_events_from(
        &self,
        urls: Vec<String>,
        filter: &Filter,
        timeout: Duration,
    ) -> Result<Events> {
        Ok(self
            .inner
            .fetch_events_from(urls, filter.deref().clone(), timeout)
            .await?
            .into())
    }

    /// Get events both from database and relays
    ///
    /// This is an auto-closing subscription and will be closed automatically on `EOSE`.
    ///
    /// You can obtain the same result by merging the `Events` from different type of sources.
    ///
    /// This method will be deprecated in the future!
    /// This is a temporary solution for who still want to query events both from database and relays and merge the result.
    /// The optimal solution is to execute a [`Client::sync`] to get all old events, [`Client::subscribe`] to get all
    /// new future events, [`NostrDatabase::query`] to query events and [`Client::handle_notifications`] to listen-for/handle new events (i.e. to know when update the UI).
    /// This will allow very fast queries, low bandwidth usage (depending on how many events the client have to sync) and a low load on relays.
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    pub async fn fetch_combined_events(
        &self,
        filter: &Filter,
        timeout: Duration,
    ) -> Result<Events> {
        Ok(self
            .inner
            .fetch_combined_events(filter.deref().clone(), timeout)
            .await?
            .into())
    }

    pub async fn send_msg_to(&self, urls: Vec<String>, msg: &ClientMessage) -> Result<Output> {
        Ok(self
            .inner
            .send_msg_to(urls, msg.deref().clone())
            .await?
            .into())
    }

    /// Send event
    ///
    /// Send event to all relays with `WRITE` flag.
    /// If `gossip` is enabled (see `Options`) the event will be sent also to NIP65 relays (automatically discovered).
    pub async fn send_event(&self, event: &Event) -> Result<SendEventOutput> {
        Ok(self.inner.send_event(event.deref()).await?.into())
    }

    /// Send event to specific relays.
    pub async fn send_event_to(&self, urls: Vec<String>, event: &Event) -> Result<SendEventOutput> {
        Ok(self.inner.send_event_to(urls, event.deref()).await?.into())
    }

    /// Signs the `EventBuilder` into an `Event` using the `NostrSigner`
    pub async fn sign_event_builder(&self, builder: &EventBuilder) -> Result<Event> {
        Ok(self
            .inner
            .sign_event_builder(builder.deref().clone())
            .await?
            .into())
    }

    /// Take an `EventBuilder`, sign it by using the `NostrSigner` and broadcast to relays (check `send_event` method for more details)
    ///
    /// Rise an error if the `NostrSigner` is not set.
    pub async fn send_event_builder(&self, builder: &EventBuilder) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event_builder(builder.deref().clone())
            .await?
            .into())
    }

    /// Take an `EventBuilder`, sign it by using the `NostrSigner` and broadcast to specific relays.
    ///
    /// Rise an error if the `NostrSigner` is not set.
    pub async fn send_event_builder_to(
        &self,
        urls: Vec<String>,
        builder: &EventBuilder,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event_builder_to(urls, builder.deref().clone())
            .await?
            .into())
    }

    /// Fetch the newest public key metadata from relays.
    ///
    /// Returns `None` if the `Metadata` of the `PublicKey` has not been found.
    ///
    /// Check `Client::fetch_events` for more details.
    ///
    /// If you only want to consult cached data,
    /// consider `client.database().profile(PUBKEY)`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub async fn fetch_metadata(
        &self,
        public_key: &PublicKey,
        timeout: Duration,
    ) -> Result<Option<Arc<Metadata>>> {
        Ok(self
            .inner
            .fetch_metadata(**public_key, timeout)
            .await?
            .map(|m| Arc::new(m.into())))
    }

    pub async fn set_metadata(&self, metadata: &Metadata) -> Result<SendEventOutput> {
        Ok(self.inner.set_metadata(metadata.deref()).await?.into())
    }

    /// Send a private direct message
    ///
    /// If gossip is enabled, the message will be sent to the NIP17 relays (automatically discovered).
    /// If gossip is not enabled will be sent to all relays with WRITE` relay service flag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[uniffi::method(default(rumor_extra_tags = []))]
    pub async fn send_private_msg(
        &self,
        receiver: &PublicKey,
        message: String,
        rumor_extra_tags: Vec<Arc<Tag>>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_private_msg(
                **receiver,
                message,
                rumor_extra_tags
                    .into_iter()
                    .map(|t| t.as_ref().deref().clone()),
            )
            .await?
            .into())
    }

    /// Send private direct message to specific relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[uniffi::method(default(rumor_extra_tags = []))]
    pub async fn send_private_msg_to(
        &self,
        urls: Vec<String>,
        receiver: &PublicKey,
        message: String,
        rumor_extra_tags: Vec<Arc<Tag>>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_private_msg_to(
                urls,
                **receiver,
                message,
                rumor_extra_tags
                    .into_iter()
                    .map(|t| t.as_ref().deref().clone()),
            )
            .await?
            .into())
    }

    /// Construct Gift Wrap and send to relays
    ///
    /// Check `send_event` method to know how sending events works.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    pub async fn gift_wrap(
        &self,
        receiver: &PublicKey,
        rumor: &UnsignedEvent,
        extra_tags: Vec<Arc<Tag>>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .gift_wrap(
                receiver.deref(),
                rumor.deref().clone(),
                extra_tags.into_iter().map(|t| t.as_ref().deref().clone()),
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
        rumor: &UnsignedEvent,
        extra_tags: Vec<Arc<Tag>>,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .gift_wrap_to(
                urls,
                receiver.deref(),
                rumor.deref().clone(),
                extra_tags.into_iter().map(|t| t.as_ref().deref().clone()),
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
