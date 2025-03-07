// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use js_sys::Array;
use nostr_sdk::async_utility::task;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod builder;
pub mod options;
pub mod output;

pub use self::builder::JsClientBuilder;
use self::output::{JsOutput, JsReconciliationOutput, JsSendEventOutput, JsSubscribeOutput};
use crate::abortable::JsAbortHandle;
use crate::database::{JsEvents, JsNostrDatabase};
use crate::duration::JsDuration;
use crate::error::{into_err, Result};
use crate::protocol::event::unsigned::JsUnsignedEvent;
use crate::protocol::event::{JsEvent, JsEventBuilder, JsTag};
use crate::protocol::filter::JsFilter;
use crate::protocol::key::JsPublicKey;
use crate::protocol::message::{JsClientMessage, JsRelayMessage};
use crate::protocol::nips::nip01::JsMetadata;
use crate::protocol::nips::nip59::JsUnwrappedGift;
use crate::relay::options::{JsRelayOptions, JsSubscribeAutoCloseOptions, JsSyncOptions};
use crate::relay::{JsRelay, JsRelayArray};
use crate::signer::JsNostrSigner;

#[wasm_bindgen(js_name = Client)]
pub struct JsClient {
    inner: Client,
}

impl From<Client> for JsClient {
    fn from(inner: Client) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Client)]
impl JsClient {
    #[wasm_bindgen(constructor)]
    pub fn new(signer: Option<JsNostrSigner>) -> Self {
        Self {
            inner: match signer {
                Some(signer) => Client::new(signer.deref().clone()),
                None => Client::default(),
            },
        }
    }

    /// Construct `ClientBuilder`
    #[inline]
    pub fn builder() -> JsClientBuilder {
        JsClientBuilder::new()
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    #[wasm_bindgen(js_name = automaticAuthentication)]
    pub fn automatic_authentication(&self, enable: bool) {
        self.inner.automatic_authentication(enable);
    }

    /// Get current nostr signer
    ///
    /// Rise error if it not set.
    pub async fn signer(&self) -> Result<JsNostrSigner> {
        Ok(self.inner.signer().await.map_err(into_err)?.into())
    }

    #[wasm_bindgen(getter)]
    pub fn database(&self) -> JsNostrDatabase {
        self.inner.database().clone().into()
    }

    /// Completely shutdown `Client`
    pub async fn shutdown(&self) {
        self.inner.shutdown().await
    }

    /// Get relays with `READ` or `WRITE` flags
    pub async fn relays(&self) -> JsRelayArray {
        self.inner
            .relays()
            .await
            .into_values()
            .map(|relay| {
                let e: JsRelay = relay.into();
                JsValue::from(e)
            })
            .collect::<Array>()
            .unchecked_into()
    }

    /// Get a previously added `Relay`
    pub async fn relay(&self, url: &str) -> Result<JsRelay> {
        Ok(self.inner.relay(url).await.map_err(into_err)?.into())
    }

    /// Add new relay
    ///
    /// Relays added with this method will have both `READ` and `WRITE` flags enabled
    ///
    /// If the relay already exists, the flags will be updated and `false` returned.
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribeTo` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    ///
    /// This method use previously set or default `Options` to configure the `Relay` (ex. set proxy, set min POW, set relay limits, ...).
    ///
    /// Connection is **NOT** automatically started with relay, remember to call `connect` method!
    #[wasm_bindgen(js_name = addRelay)]
    pub async fn add_relay(&self, url: String) -> Result<bool> {
        self.inner.add_relay(url).await.map_err(into_err)
    }

    #[wasm_bindgen(js_name = addRelayWithOpts)]
    pub async fn add_relay_with_opts(&self, url: &str, opts: &JsRelayOptions) -> Result<bool> {
        self.inner
            .pool()
            .add_relay(url, opts.deref().clone())
            .await
            .map_err(into_err)
    }

    /// Add discovery relay
    ///
    /// If relay already exists, this method automatically add the `DISCOVERY` flag to it and return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[wasm_bindgen(js_name = addDiscoveryRelay)]
    pub async fn add_discovery_relay(&self, url: String) -> Result<bool> {
        self.inner.add_discovery_relay(url).await.map_err(into_err)
    }

    /// Add read relay
    ///
    /// If relay already exists, this method add the `READ` flag to it and return `false`.
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    #[wasm_bindgen(js_name = addReadRelay)]
    pub async fn add_read_relay(&self, url: String) -> Result<bool> {
        self.inner.add_read_relay(url).await.map_err(into_err)
    }

    /// Add write relay
    ///
    /// If relay already exists, this method add the `WRITE` flag to it and return `false`.
    #[wasm_bindgen(js_name = addWriteRelay)]
    pub async fn add_write_relay(&self, url: String) -> Result<bool> {
        self.inner.add_write_relay(url).await.map_err(into_err)
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has `GOSSIP` flag, it will not be removed from the pool and its
    /// flags will be updated (remove `READ`, `WRITE` and `DISCOVERY` flags).
    #[wasm_bindgen(js_name = removeRelay)]
    pub async fn remove_relay(&self, url: &str) -> Result<()> {
        self.inner.remove_relay(url).await.map_err(into_err)
    }

    /// Force remove and disconnect relay
    ///
    /// Note: this method will remove the relay, also if it's in use for the gossip model or other service!
    #[wasm_bindgen(js_name = forceRemoveRelay)]
    pub async fn force_remove_relay(&self, url: &str) -> Result<()> {
        self.inner.force_remove_relay(url).await.map_err(into_err)
    }

    /// Disconnect and remove all relays
    ///
    /// Some relays used by some services could not be disconnected with this method
    /// (like the ones used for gossip).
    /// Use [`Client::force_remove_all_relays`] to remove every relay.
    #[wasm_bindgen(js_name = removeAllRelays)]
    pub async fn remove_all_relays(&self) {
        self.inner.remove_all_relays().await
    }

    /// Disconnect and force remove all relays
    #[wasm_bindgen(js_name = forceRemoveAllRelays)]
    pub async fn force_remove_all_relays(&self) {
        self.inner.force_remove_all_relays().await
    }

    /// Connect to a previously added relay
    #[wasm_bindgen(js_name = connectRelay)]
    pub async fn connect_relay(&self, url: String) -> Result<()> {
        self.inner.connect_relay(url).await.map_err(into_err)
    }

    /// Disconnect relay
    #[wasm_bindgen(js_name = disconnectRelay)]
    pub async fn disconnect_relay(&self, url: String) -> Result<()> {
        self.inner.disconnect_relay(url).await.map_err(into_err)
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
    #[wasm_bindgen(js_name = waitForConnection)]
    pub async fn wait_for_connection(&self, timeout: &JsDuration) {
        self.inner.wait_for_connection(**timeout).await
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
    #[wasm_bindgen(js_name = tryConnect)]
    pub async fn try_connect(&self, timeout: &JsDuration) -> JsOutput {
        self.inner.try_connect(**timeout).await.into()
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) {
        self.inner.disconnect().await
    }

    /// Subscribe to filters
    ///
    /// If `gossip` is enabled (see `Options]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    pub async fn subscribe(
        &self,
        filter: &JsFilter,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) -> Result<JsSubscribeOutput> {
        self.inner
            .subscribe(filter.deref().clone(), opts.map(|o| *o))
            .await
            .map_err(into_err)
            .map(|o| o.into())
    }

    /// Subscribe to filters with custom subscription ID
    ///
    /// If `gossip` is enabled (see `Options]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[wasm_bindgen(js_name = subscribeWithId)]
    pub async fn subscribe_with_id(
        &self,
        id: &str,
        filter: &JsFilter,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) -> Result<JsOutput> {
        self.inner
            .subscribe_with_id(
                SubscriptionId::new(id),
                filter.deref().clone(),
                opts.map(|o| *o),
            )
            .await
            .map_err(into_err)
            .map(|o| o.into())
    }

    /// Subscribe to filters to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[wasm_bindgen(js_name = subscribeTo)]
    pub async fn subscribe_to(
        &self,
        urls: Vec<String>,
        filter: &JsFilter,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) -> Result<JsSubscribeOutput> {
        self.inner
            .subscribe_to(urls, filter.deref().clone(), opts.map(|o| *o))
            .await
            .map_err(into_err)
            .map(|o| o.into())
    }

    /// Subscribe to filters with custom subscription ID to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[wasm_bindgen(js_name = subscribeWithIdTo)]
    pub async fn subscribe_with_id_to(
        &self,
        urls: Vec<String>,
        id: &str,
        filter: &JsFilter,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) -> Result<JsOutput> {
        self.inner
            .subscribe_with_id_to(
                urls,
                SubscriptionId::new(id),
                filter.deref().clone(),
                opts.map(|o| *o),
            )
            .await
            .map_err(into_err)
            .map(|o| o.into())
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self, subscription_id: &str) {
        self.inner
            .unsubscribe(&SubscriptionId::new(subscription_id))
            .await;
    }

    /// Unsubscribe
    #[wasm_bindgen(js_name = unsubscribeAll)]
    pub async fn unsubscribe_all(&self) {
        self.inner.unsubscribe_all().await;
    }

    /// Sync events with relays (negentropy reconciliation)
    ///
    /// If `gossip` is enabled (see `Options`) the events will be reconciled also with
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// <https://github.com/hoytech/negentropy>
    pub async fn sync(
        &self,
        filter: &JsFilter,
        opts: &JsSyncOptions,
    ) -> Result<JsReconciliationOutput> {
        self.inner
            .sync(filter.deref().clone(), opts.deref())
            .await
            .map_err(into_err)
            .map(|o| o.into())
    }

    /// Fetch events from relays
    ///
    /// This is an auto-closing subscription and will be closed automatically on `EOSE`.
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see `Options`) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    #[wasm_bindgen(js_name = fetchEvents)]
    pub async fn fetch_events(&self, filter: &JsFilter, timeout: &JsDuration) -> Result<JsEvents> {
        let events: Events = self
            .inner
            .fetch_events(filter.deref().clone(), **timeout)
            .await
            .map_err(into_err)?;
        Ok(events.into())
    }

    /// Fetch events from specific relays
    ///
    /// This is an auto-closing subscription and will be closed automatically on `EOSE`.
    #[wasm_bindgen(js_name = fetchEventsFrom)]
    pub async fn fetch_events_from(
        &self,
        urls: Vec<String>,
        filter: &JsFilter,
        timeout: &JsDuration,
    ) -> Result<JsEvents> {
        let events: Events = self
            .inner
            .fetch_events_from(urls, filter.deref().clone(), **timeout)
            .await
            .map_err(into_err)?;
        Ok(events.into())
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
    /// This will allow very fast queries, low bandwidth usage (depending on how many events the client have to reconcile) and a lower load on the relays.
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    #[wasm_bindgen(js_name = fetchCombinedEvents)]
    pub async fn fetch_combined_events(
        &self,
        filter: &JsFilter,
        timeout: &JsDuration,
    ) -> Result<JsEvents> {
        let events: Events = self
            .inner
            .fetch_combined_events(filter.deref().clone(), **timeout)
            .await
            .map_err(into_err)?;
        Ok(events.into())
    }

    /// Send client message to a specific relay
    #[wasm_bindgen(js_name = sendMsgTo)]
    pub async fn send_msg_to(&self, urls: Vec<String>, msg: &JsClientMessage) -> Result<JsOutput> {
        self.inner
            .send_msg_to(urls, msg.deref().clone())
            .await
            .map_err(into_err)
            .map(Into::into)
    }

    /// Send event
    ///
    /// Send event to all relays with `WRITE` flag.
    /// If `gossip` is enabled (see `Options`) the event will be sent also to NIP65 relays (automatically discovered).
    #[wasm_bindgen(js_name = sendEvent)]
    pub async fn send_event(&self, event: &JsEvent) -> Result<JsSendEventOutput> {
        self.inner
            .send_event(event.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send event to specific relay
    #[wasm_bindgen(js_name = sendEventTo)]
    pub async fn send_event_to(
        &self,
        urls: Vec<String>,
        event: &JsEvent,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .send_event_to(urls, event.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Signs the `EventBuilder` into an `Event` using the `NostrSigner`
    #[wasm_bindgen(js_name = signEventBuilder)]
    pub async fn sign_event_builder(&self, builder: &JsEventBuilder) -> Result<JsEvent> {
        self.inner
            .sign_event_builder(builder.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Take an `EventBuilder`, sign it by using the `NostrSigner` and broadcast to relays (check `send_event` method for more details)
    ///
    /// Rise an error if the `NostrSigner` is not set.
    #[wasm_bindgen(js_name = sendEventBuilder)]
    pub async fn send_event_builder(&self, builder: &JsEventBuilder) -> Result<JsSendEventOutput> {
        self.inner
            .send_event_builder(builder.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Take an `EventBuilder`, sign it by using the `NostrSigner` and broadcast to specific relays.
    ///
    /// Rise an error if the `NostrSigner` is not set.
    #[wasm_bindgen(js_name = sendEventBuilderTo)]
    pub async fn send_event_builder_to(
        &self,
        urls: Vec<String>,
        builder: &JsEventBuilder,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .send_event_builder_to(urls, builder.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Fetch the newest public key metadata from relays.
    ///
    /// Returns `None` if the `Metadata` of the `PublicKey` has not been found.
    ///
    /// Check `Client.fetchEvents` for more details.
    ///
    /// If you only want to consult cached data,
    /// consider `client.database().profile(PUBKEY)`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = fetchMetadata)]
    pub async fn fetch_metadata(
        &self,
        public_key: &JsPublicKey,
        timeout: &JsDuration,
    ) -> Result<Option<JsMetadata>> {
        Ok(self
            .inner
            .fetch_metadata(**public_key, **timeout)
            .await
            .map_err(into_err)?
            .map(|m| m.into()))
    }

    /// Update metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = setMetadata)]
    pub async fn set_metadata(&self, metadata: &JsMetadata) -> Result<JsSendEventOutput> {
        self.inner
            .set_metadata(metadata.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send a private direct message
    ///
    /// If gossip is enabled, the message will be sent to the NIP17 relays (automatically discovered).
    /// If gossip is not enabled will be sent to all relays with WRITE` relay service flag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[wasm_bindgen(js_name = sendPrivateMsg)]
    pub async fn send_private_msg(
        &self,
        receiver: &JsPublicKey,
        message: &str,
        rumor_extra_tags: Option<Vec<JsTag>>,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .send_private_msg(
                **receiver,
                message,
                rumor_extra_tags
                    .unwrap_or_default()
                    .into_iter()
                    .map(|t| t.inner),
            )
            .await
            .map_err(into_err)
            .map(Into::into)
    }

    /// Send private direct message to specific relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[wasm_bindgen(js_name = sendPrivateMsgTo)]
    pub async fn send_private_msg_to(
        &self,
        urls: Vec<String>,
        receiver: &JsPublicKey,
        message: &str,
        rumor_extra_tags: Option<Vec<JsTag>>,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .send_private_msg_to(
                urls,
                **receiver,
                message,
                rumor_extra_tags
                    .unwrap_or_default()
                    .into_iter()
                    .map(|t| t.inner),
            )
            .await
            .map_err(into_err)
            .map(Into::into)
    }

    /// Construct Gift Wrap and send to relays
    ///
    /// Check `sendEvent` method to know how sending events works.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = giftWrap)]
    pub async fn gift_wrap(
        &self,
        receiver: &JsPublicKey,
        rumor: &JsUnsignedEvent,
        extra_tags: Option<Vec<JsTag>>,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .gift_wrap(
                receiver.deref(),
                rumor.deref().clone(),
                extra_tags.unwrap_or_default().into_iter().map(|t| t.inner),
            )
            .await
            .map_err(into_err)
            .map(Into::into)
    }

    /// Construct Gift Wrap and send to specific relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = giftWrapTo)]
    pub async fn gift_wrap_to(
        &self,
        urls: Vec<String>,
        receiver: &JsPublicKey,
        rumor: &JsUnsignedEvent,
        extra_tags: Option<Vec<JsTag>>,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .gift_wrap_to(
                urls,
                receiver.deref(),
                rumor.deref().clone(),
                extra_tags.unwrap_or_default().into_iter().map(|t| t.inner),
            )
            .await
            .map_err(into_err)
            .map(Into::into)
    }

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = unwrapGiftWrap)]
    pub async fn unwrap_gift_wrap(&self, gift_wrap: &JsEvent) -> Result<JsUnwrappedGift> {
        Ok(self
            .inner
            .unwrap_gift_wrap(gift_wrap.deref())
            .await
            .map_err(into_err)?
            .into())
    }

    /// Handle notifications
    ///
    /// **This method spawn a task**, so ensure to keep up the app after calling this (if needed).
    ///
    /// To exit from the handle notifications loop, return `true` or call `abortable.abort();`.
    ///
    /// # Example
    /// ```javascript
    /// // Subscribe to filters
    /// const filter = new Filter().author(keys.publicKey);
    /// await client.subscribe([filter]);
    ///
    /// const handle = {
    ///    // Handle event
    ///    handleEvent: async (relayUrl, subscriptionId, event) => {
    ///        console.log("Received new event from", relayUrl);
    ///        if (event.kind == 4) {
    ///            try {
    ///                let content = nip04Decrypt(keys.secretKey, event.author, event.content);
    ///                console.log("Message:", content);
    ///                await client.sendDirectMsg(event.author, "Echo: " + content);
    ///
    ///                if (content == "stop") {
    ///                    return true;
    ///                }
    ///            } catch (error) {
    ///                console.log("Impossible to decrypt DM:", error);
    ///            }
    ///         }
    ///     },
    ///     // Handle relay message
    ///     handleMsg: async (relayUrl, message) => {
    ///         console.log("Received message from", relayUrl, message.asJson());
    ///     }
    ///  };
    ///
    /// let abortable = client.handleNotifications(handle);
    /// // Optionally, call `abortable.abort();` when you need to stop handle notifications task
    /// ```
    #[wasm_bindgen(js_name = handleNotifications)]
    pub fn handle_notifications(&self, callback: HandleNotification) -> Result<JsAbortHandle> {
        let inner = self.inner.clone();
        let handle = task::abortable(async move {
            inner
            .handle_notifications(|notification| async {
                match notification {
                    RelayPoolNotification::Message { relay_url, message } => {
                        let message: JsRelayMessage = message.into();
                        if callback.handle_msg(relay_url.to_string(), message).await.as_bool().unwrap_or_default() {
                            tracing::info!("Received `true` in `handlemsg`: exiting from `handleNotifications`");
                            return Ok(true);
                        }
                    }
                    RelayPoolNotification::Event { relay_url, subscription_id, event } => {
                        let event: JsEvent = (*event).into();
                        if callback.handle_event(relay_url.to_string(), subscription_id.to_string(), event).await.as_bool().unwrap_or_default() {
                            tracing::info!("Received `true` in `handleEvent`: exiting from `handleNotifications`");
                            return Ok(true);
                        }
                    }
                    _ => (),
                }
                Ok(false)
            })
            .await
            .map_err(into_err).unwrap();
        });
        Ok(handle.into())
    }
}

#[wasm_bindgen(typescript_custom_section)]
const HANDLE_NOTIFICATION: &'static str = r#"
interface HandleNotification {
    handleEvent: (relayUrl: string, subscriptionId: string, event: Event) => Promise<boolean>;
    handleMsg: (relayUrl: string, message: RelayMessage) => Promise<boolean>;
}
"#;

#[wasm_bindgen]
extern "C" {
    /// Handle notification
    #[wasm_bindgen(typescript_type = "HandleNotification")]
    pub type HandleNotification;

    /// handle event
    #[wasm_bindgen(structural, method, js_name = handleEvent)]
    pub async fn handle_event(
        this: &HandleNotification,
        relay_url: String,
        subscription_id: String,
        event: JsEvent,
    ) -> JsValue;

    /// Handle message
    #[wasm_bindgen(structural, method, js_name = handleMsg)]
    pub async fn handle_msg(
        this: &HandleNotification,
        relay_url: String,
        message: JsRelayMessage,
    ) -> JsValue;
}
