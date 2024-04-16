// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::time::Duration;

use js_sys::Array;
use nostr_js::error::{into_err, Result};
use nostr_js::event::{JsEvent, JsEventArray, JsEventBuilder, JsEventId, JsTag};
use nostr_js::key::JsPublicKey;
use nostr_js::message::{JsClientMessage, JsRelayMessage};
use nostr_js::types::{JsContact, JsFilter, JsMetadata, JsTimestamp};
use nostr_sdk::async_utility::thread;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod builder;
pub mod options;
pub mod signer;
pub mod zapper;

pub use self::builder::JsClientBuilder;
use self::options::JsOptions;
pub use self::signer::JsNostrSigner;
use self::zapper::{JsZapDetails, JsZapEntity};
use crate::abortable::JsAbortHandle;
use crate::database::JsNostrDatabase;
use crate::duration::JsDuration;
use crate::relay::options::{JsNegentropyOptions, JsSubscribeAutoCloseOptions};
use crate::relay::{JsRelay, JsRelayArray};

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
        Self::with_opts(signer, &JsOptions::new())
    }

    /// Create a new Client with Options
    #[wasm_bindgen(js_name = withOpts)]
    pub fn with_opts(signer: Option<JsNostrSigner>, opts: &JsOptions) -> Self {
        Self {
            inner: match signer {
                Some(signer) => Client::with_opts(signer.deref().clone(), opts.deref().clone()),
                None => Client::builder().opts(opts.deref().clone()).build(),
            },
        }
    }

    /// Construct `ClientBuilder`
    #[inline]
    pub fn builder() -> JsClientBuilder {
        JsClientBuilder::new()
    }

    /// Update default difficulty for new `Event`
    #[wasm_bindgen(js_name = updateDifficulty)]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    /// Get current nostr signer
    ///
    /// Rise error if it not set.
    pub async fn signer(&self) -> Result<JsNostrSigner> {
        Ok(self.inner.signer().await.map_err(into_err)?.into())
    }

    #[wasm_bindgen(getter)]
    pub fn database(&self) -> JsNostrDatabase {
        self.inner.database().into()
    }

    /// Completely shutdown `Client`
    pub async fn shutdown(self) -> Result<()> {
        self.inner.shutdown().await.map_err(into_err)
    }

    /// Get relays
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
    /// Return `false` if the relay already exists.
    ///
    /// This method use perviously set or default `Options` to configure the `Relay` (ex. set proxy, set min POW, set relay limits, ...).
    ///
    /// Connection is **NOT** automatically started with relay, remember to call `connect` method!
    #[wasm_bindgen(js_name = addRelay)]
    pub async fn add_relay(&self, url: String) -> Result<bool> {
        self.inner.add_relay(url).await.map_err(into_err)
    }

    /// Add multiple relays
    ///
    /// Connection is **NOT** automatically started with relays, remember to call `connect` method!
    #[wasm_bindgen(js_name = addRelays)]
    pub async fn add_relays(&self, urls: Vec<String>) -> Result<()> {
        self.inner.add_relays(urls).await.map_err(into_err)
    }

    /// Remove relay
    #[wasm_bindgen(js_name = removeRelay)]
    pub async fn remove_relay(&self, url: String) -> Result<()> {
        self.inner.remove_relay(url).await.map_err(into_err)
    }

    /// Connect relay
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
    pub async fn connect(&self) {
        self.inner.connect().await;
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) -> Result<()> {
        self.inner.disconnect().await.map_err(into_err)
    }

    /// Subscribe to filters
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    pub async fn subscribe(
        &self,
        filters: Vec<JsFilter>,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) -> String {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        self.inner
            .subscribe(filters, opts.map(|o| *o))
            .await
            .to_string()
    }

    /// Subscribe to filters with custom subscription ID
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    #[wasm_bindgen(js_name = subscribeWithId)]
    pub async fn subscribe_with_id(
        &self,
        id: &str,
        filters: Vec<JsFilter>,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        self.inner
            .subscribe_with_id(SubscriptionId::new(id), filters, opts.map(|o| *o))
            .await
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self, subscription_id: &str) {
        self.inner
            .unsubscribe(SubscriptionId::new(subscription_id))
            .await;
    }

    /// Unsubscribe
    #[wasm_bindgen(js_name = unsubscribeAll)]
    pub async fn unsubscribe_all(&self) {
        self.inner.unsubscribe_all().await;
    }

    /// Get events of filters
    ///
    /// If timeout is not set, the default one from Options will be used.
    #[wasm_bindgen(js_name = getEventsOf)]
    pub async fn get_events_of(
        &self,
        filters: Vec<JsFilter>,
        timeout: Option<JsDuration>,
    ) -> Result<JsEventArray> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        let timeout: Option<Duration> = timeout.map(|d| *d);
        let events: Vec<Event> = self
            .inner
            .get_events_of(filters, timeout)
            .await
            .map_err(into_err)?;
        let events: JsEventArray = events
            .into_iter()
            .map(|e| {
                let e: JsEvent = e.into();
                JsValue::from(e)
            })
            .collect::<Array>()
            .unchecked_into();
        Ok(events)
    }

    /// Get events of filters from specific relays
    ///
    /// Get events both from **local database** and **relays**
    #[wasm_bindgen(js_name = getEventsFrom)]
    pub async fn get_events_from(
        &self,
        urls: Vec<String>,
        filters: Vec<JsFilter>,
        timeout: Option<JsDuration>,
    ) -> Result<JsEventArray> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        let timeout: Option<Duration> = timeout.map(|d| *d);
        let events: Vec<Event> = self
            .inner
            .get_events_from(urls, filters, timeout)
            .await
            .map_err(into_err)?;
        let events: JsEventArray = events
            .into_iter()
            .map(|e| {
                let e: JsEvent = e.into();
                JsValue::from(e)
            })
            .collect::<Array>()
            .unchecked_into();
        Ok(events)
    }

    /// Send client message
    #[wasm_bindgen(js_name = sendMsg)]
    pub async fn send_msg(&self, msg: &JsClientMessage) -> Result<()> {
        self.inner
            .send_msg(msg.deref().clone())
            .await
            .map_err(into_err)
    }

    /// Send client message to a specific relay
    #[wasm_bindgen(js_name = sendMsgTo)]
    pub async fn send_msg_to(&self, urls: Vec<String>, msg: &JsClientMessage) -> Result<()> {
        self.inner
            .send_msg_to(urls, msg.deref().clone())
            .await
            .map_err(into_err)
    }

    /// Send event
    ///
    /// This method will wait for the `OK` message from the relay.
    /// If you not want to wait for the `OK` message, use `sendMsg` method instead.
    #[wasm_bindgen(js_name = sendEvent)]
    pub async fn send_event(&self, event: &JsEvent) -> Result<JsEventId> {
        self.inner
            .send_event(event.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send event to specific relay
    ///
    /// This method will wait for the `OK` message from the relay.
    /// If you not want to wait for the `OK` message, use `sendMsgTo` method instead.
    #[wasm_bindgen(js_name = sendEventTo)]
    pub async fn send_event_to(&self, urls: Vec<String>, event: &JsEvent) -> Result<JsEventId> {
        self.inner
            .send_event_to(urls, event.deref().clone())
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

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to all relays.
    ///
    /// Rise an error if the [`NostrSigner`] is not set.
    #[wasm_bindgen(js_name = sendEventBuilder)]
    pub async fn send_event_builder(&self, builder: &JsEventBuilder) -> Result<JsEventId> {
        self.inner
            .send_event_builder(builder.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to specific relays.
    ///
    /// Rise an error if the [`NostrSigner`] is not set.
    #[wasm_bindgen(js_name = sendEventBuilderTo)]
    pub async fn send_event_builder_to(
        &self,
        urls: Vec<String>,
        builder: &JsEventBuilder,
    ) -> Result<JsEventId> {
        self.inner
            .send_event_builder_to(urls, builder.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Update metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = setMetadata)]
    pub async fn set_metadata(&self, metadata: &JsMetadata) -> Result<JsEventId> {
        self.inner
            .set_metadata(metadata.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Publish text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = publishTextNote)]
    pub async fn publish_text_note(&self, content: String, tags: Vec<JsTag>) -> Result<JsEventId> {
        self.inner
            .publish_text_note(content, tags.into_iter().map(|t| t.into()))
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Set contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[wasm_bindgen(js_name = setContactList)]
    pub async fn set_contact_list(&self, list: Vec<JsContact>) -> Result<JsEventId> {
        let list = list.into_iter().map(|c| c.inner());
        self.inner
            .set_contact_list(list)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send encrypted direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/04.md>
    #[wasm_bindgen(js_name = sendDirectMsg)]
    pub async fn send_direct_msg(
        &self,
        receiver: &JsPublicKey,
        msg: &str,
        reply: Option<JsEventId>,
    ) -> Result<JsEventId> {
        self.inner
            .send_direct_msg(**receiver, msg, reply.map(|id| id.into()))
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Repost
    pub async fn repost(&self, event: &JsEvent, relay_url: Option<String>) -> Result<JsEventId> {
        self.inner
            .repost(event.deref(), relay_url.map(UncheckedUrl::from))
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Delete event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    #[wasm_bindgen(js_name = deleteEvent)]
    pub async fn delete_event(&self, event_id: &JsEventId) -> Result<JsEventId> {
        self.inner
            .delete_event(**event_id)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Like event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn like(&self, event: &JsEvent) -> Result<JsEventId> {
        self.inner
            .like(event.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Disike event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn dislike(&self, event: &JsEvent) -> Result<JsEventId> {
        self.inner
            .dislike(event.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// React to an [`Event`]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn reaction(&self, event: &JsEvent, reaction: &str) -> Result<JsEventId> {
        self.inner
            .reaction(event.deref(), reaction)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[wasm_bindgen(js_name = newChannel)]
    pub async fn new_channel(&self, metadata: &JsMetadata) -> Result<JsEventId> {
        self.inner
            .new_channel(metadata.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Update channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[wasm_bindgen(js_name = setChannelMetadata)]
    pub async fn set_channel_metadata(
        &self,
        channel_id: &JsEventId,
        relay_url: Option<String>,
        metadata: &JsMetadata,
    ) -> Result<JsEventId> {
        let relay_url: Option<Url> = match relay_url {
            Some(relay_url) => Some(Url::parse(&relay_url).map_err(into_err)?),
            None => None,
        };
        self.inner
            .set_channel_metadata(**channel_id, relay_url, metadata.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send message to channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[wasm_bindgen(js_name = sendChannelMsg)]
    pub async fn send_channel_msg(
        &self,
        channel_id: &JsEventId,
        relay_url: &str,
        msg: &str,
    ) -> Result<JsEventId> {
        let relay_url: Url = Url::parse(relay_url).map_err(into_err)?;
        self.inner
            .send_channel_msg(**channel_id, relay_url, msg)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Hide channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[wasm_bindgen(js_name = hideChannelUser)]
    pub async fn hide_channel_msg(
        &self,
        message_id: &JsEventId,
        reason: Option<String>,
    ) -> Result<JsEventId> {
        self.inner
            .hide_channel_msg(**message_id, reason)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[wasm_bindgen(js_name = muteChannelUser)]
    pub async fn mute_channel_user(
        &self,
        pubkey: &JsPublicKey,
        reason: Option<String>,
    ) -> Result<JsEventId> {
        self.inner
            .mute_channel_user(**pubkey, reason)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send a Zap!
    ///
    /// This method automatically create a split zap to support Rust Nostr development.
    pub async fn zap(
        &self,
        to: &JsZapEntity,
        satoshi: f64,
        details: Option<JsZapDetails>,
    ) -> Result<()> {
        self.inner
            .zap(**to, satoshi as u64, details.map(|d| d.into()))
            .await
            .map_err(into_err)
    }

    /// Gift Wrap
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = giftWrap)]
    pub async fn gift_wrap(
        &self,
        receiver: &JsPublicKey,
        rumor: &JsEventBuilder,
        expiration: Option<JsTimestamp>,
    ) -> Result<()> {
        self.inner
            .gift_wrap(**receiver, rumor.deref().clone(), expiration.map(|t| *t))
            .await
            .map_err(into_err)
    }

    /// Send GiftWrapper Sealed Direct message
    #[wasm_bindgen(js_name = sendSealedMsg)]
    pub async fn send_sealed_msg(
        &self,
        receiver: &JsPublicKey,
        message: &str,
        expiration: Option<JsTimestamp>,
    ) -> Result<()> {
        self.inner
            .send_sealed_msg(**receiver, message, expiration.map(|t| *t))
            .await
            .map_err(into_err)
    }

    /// Negentropy reconciliation
    ///
    /// <https://github.com/hoytech/negentropy>
    pub async fn reconcile(&self, filter: &JsFilter, opts: &JsNegentropyOptions) -> Result<()> {
        self.inner
            .reconcile(filter.deref().clone(), **opts)
            .await
            .map_err(into_err)
    }

    /// Handle notifications
    ///
    /// **This method spawn a thread**, so ensure to keep up the app after calling this (if needed).
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
    /// // Optionally, call `abortable.abort();` when you need to stop handle notifications thread
    /// ```
    #[wasm_bindgen(js_name = handleNotifications)]
    pub fn handle_notifications(&self, callback: HandleNotification) -> Result<JsAbortHandle> {
        let inner = self.inner.clone();
        let handle = thread::abortable(async move {
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
        }).map_err(into_err)?;
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
    #[wasm_bindgen(typescript_type = "HandleNotification")]
    pub type HandleNotification;

    #[wasm_bindgen(structural, method, js_name = handleEvent)]
    pub async fn handle_event(
        this: &HandleNotification,
        relay_url: String,
        subscription_id: String,
        event: JsEvent,
    ) -> JsValue;

    #[wasm_bindgen(structural, method, js_name = handleMsg)]
    pub async fn handle_msg(
        this: &HandleNotification,
        relay_url: String,
        message: JsRelayMessage,
    ) -> JsValue;
}
