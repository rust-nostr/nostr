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
use nostr_js::nips::nip59::JsUnwrappedGift;
use nostr_js::types::{JsContact, JsFilter, JsMetadata, JsTimestamp};
use nostr_sdk::async_utility::thread;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod builder;
pub mod options;
pub mod signer;
pub mod zapper;

pub use self::builder::JsClientBuilder;
use self::options::{JsEventSource, JsOptions};
pub use self::signer::JsNostrSigner;
use self::zapper::{JsZapDetails, JsZapEntity};
use crate::abortable::JsAbortHandle;
use crate::database::JsNostrDatabase;
use crate::duration::JsDuration;
use crate::pool::result::{JsOutput, JsReconciliationOutput, JsSendEventOutput, JsSubscribeOutput};
use crate::pool::JsRelayPool;
use crate::relay::blacklist::JsRelayBlacklist;
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

    /// Update minimum POW difficulty for received events
    ///
    /// Events with a POW lower than the current value will be ignored to prevent resources exhaustion.
    #[inline]
    #[wasm_bindgen(js_name = updateMinPowDifficulty)]
    pub fn update_min_pow_difficulty(&self, difficulty: u8) {
        self.inner.update_min_pow_difficulty(difficulty);
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
    pub fn pool(&self) -> JsRelayPool {
        self.inner.pool().into()
    }

    #[wasm_bindgen(getter)]
    pub fn database(&self) -> JsNostrDatabase {
        self.inner.database().into()
    }

    #[wasm_bindgen(getter)]
    pub fn blacklist(&self) -> JsRelayBlacklist {
        self.inner.blacklist().into()
    }

    /// Mute event IDs
    ///
    /// Add event IDs to blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    #[wasm_bindgen(js_name = muteIds)]
    pub async fn mute_ids(&self, ids: Vec<JsEventId>) {
        self.inner.mute_ids(ids.into_iter().map(|id| *id)).await
    }

    /// Unmute event IDs
    ///
    /// Remove event IDs from blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    #[wasm_bindgen(js_name = unmuteIds)]
    pub async fn unmute_ids(&self, ids: Vec<JsEventId>) {
        self.inner.unmute_ids(ids.iter().map(|id| id.deref())).await
    }

    /// Mute public keys
    ///
    /// Add public keys to blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    #[wasm_bindgen(js_name = mutePublicKeys)]
    pub async fn mute_public_keys(&self, public_keys: Vec<JsPublicKey>) {
        self.inner
            .mute_public_keys(public_keys.into_iter().map(|p| *p))
            .await
    }

    /// Unmute public keys
    ///
    /// Remove public keys from blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    #[wasm_bindgen(js_name = unmutePublicKeys)]
    pub async fn unmute_public_keys(&self, public_keys: Vec<JsPublicKey>) {
        self.inner
            .unmute_public_keys(public_keys.iter().map(|p| p.deref()))
            .await
    }

    /// Completely shutdown `Client`
    pub async fn shutdown(&self) -> Result<()> {
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

    /// Remove relay
    #[wasm_bindgen(js_name = removeRelay)]
    pub async fn remove_relay(&self, url: String) -> Result<()> {
        self.inner.remove_relay(url).await.map_err(into_err)
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
    pub async fn connect(&self) {
        self.inner.connect().await
    }

    /// Connect to all added relays
    ///
    /// Try to connect to the relays and wait for them to be connected at most for the specified `timeout`.
    /// The code continues if the `timeout` is reached or if all relays connect.
    #[inline]
    #[wasm_bindgen(js_name = connectWithTimeout)]
    pub async fn connect_with_timeout(&self, timeout: &JsDuration) {
        self.inner.connect_with_timeout(**timeout).await
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) -> Result<()> {
        self.inner.disconnect().await.map_err(into_err)
    }

    /// Subscribe to filters to all connected relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeAutoCloseOptions`.
    pub async fn subscribe(
        &self,
        filters: Vec<JsFilter>,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) -> Result<JsSubscribeOutput> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        self.inner
            .subscribe(filters, opts.map(|o| *o))
            .await
            .map_err(into_err)
            .map(|o| o.into())
    }

    /// Subscribe to filters with custom subscription ID to all connected relays
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
    ) -> Result<JsOutput> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        self.inner
            .subscribe_with_id(SubscriptionId::new(id), filters, opts.map(|o| *o))
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
        filters: Vec<JsFilter>,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) -> Result<JsSubscribeOutput> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        self.inner
            .subscribe_to(urls, filters, opts.map(|o| *o))
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
        filters: Vec<JsFilter>,
        opts: Option<JsSubscribeAutoCloseOptions>,
    ) -> Result<JsOutput> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        self.inner
            .subscribe_with_id_to(urls, SubscriptionId::new(id), filters, opts.map(|o| *o))
            .await
            .map_err(into_err)
            .map(|o| o.into())
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
        source: &JsEventSource,
    ) -> Result<JsEventArray> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.into()).collect();
        let events: Vec<Event> = self
            .inner
            .get_events_of(filters, source.deref().clone())
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
    /// If `gossip` is enabled (see `Options`) the event will be sent also to NIP-65 relays (automatically discovered).
    #[wasm_bindgen(js_name = sendEvent)]
    pub async fn send_event(&self, event: &JsEvent) -> Result<JsSendEventOutput> {
        self.inner
            .send_event(event.deref().clone())
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

    /// Fetch the newest public key metadata from database and connected relays.
    ///
    /// If you only want to consult cached data,
    /// consider `client.database().profile(PUBKEY)`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = fetchMetadata)]
    pub async fn fetch_metadata(
        &self,
        public_key: &JsPublicKey,
        timeout: Option<JsDuration>,
    ) -> Result<JsMetadata> {
        self.inner
            .fetch_metadata(**public_key, timeout.map(|t| *t))
            .await
            .map_err(into_err)
            .map(|m| m.into())
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

    /// Publish text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = publishTextNote)]
    pub async fn publish_text_note(
        &self,
        content: String,
        tags: Vec<JsTag>,
    ) -> Result<JsSendEventOutput> {
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
    pub async fn set_contact_list(&self, list: Vec<JsContact>) -> Result<JsSendEventOutput> {
        let list = list.into_iter().map(|c| c.into());
        self.inner
            .set_contact_list(list)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send private direct message to all relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[wasm_bindgen(js_name = sendPrivateMsg)]
    pub async fn send_private_msg(
        &self,
        receiver: &JsPublicKey,
        message: &str,
        reply_to: Option<JsEventId>,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .send_private_msg(**receiver, message, reply_to.map(|t| *t))
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
        reply_to: Option<JsEventId>,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .send_private_msg_to(urls, **receiver, message, reply_to.map(|t| *t))
            .await
            .map_err(into_err)
            .map(Into::into)
    }

    /// Repost
    pub async fn repost(
        &self,
        event: &JsEvent,
        relay_url: Option<String>,
    ) -> Result<JsSendEventOutput> {
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
    pub async fn delete_event(&self, event_id: &JsEventId) -> Result<JsSendEventOutput> {
        self.inner
            .delete_event(**event_id)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Like event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn like(&self, event: &JsEvent) -> Result<JsSendEventOutput> {
        self.inner
            .like(event.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Disike event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn dislike(&self, event: &JsEvent) -> Result<JsSendEventOutput> {
        self.inner
            .dislike(event.deref())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// React to an [`Event`]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    pub async fn reaction(&self, event: &JsEvent, reaction: &str) -> Result<JsSendEventOutput> {
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
    pub async fn new_channel(&self, metadata: &JsMetadata) -> Result<JsSendEventOutput> {
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
    ) -> Result<JsSendEventOutput> {
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
    ) -> Result<JsSendEventOutput> {
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
    ) -> Result<JsSendEventOutput> {
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
    ) -> Result<JsSendEventOutput> {
        self.inner
            .mute_channel_user(**pubkey, reason)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send a Zap!
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

    /// Construct Gift Wrap and send to all relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[wasm_bindgen(js_name = giftWrap)]
    pub async fn gift_wrap(
        &self,
        receiver: &JsPublicKey,
        rumor: &JsEventBuilder,
        expiration: Option<JsTimestamp>,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .gift_wrap(
                receiver.deref(),
                rumor.deref().clone(),
                expiration.map(|t| *t),
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
        rumor: &JsEventBuilder,
        expiration: Option<JsTimestamp>,
    ) -> Result<JsSendEventOutput> {
        self.inner
            .gift_wrap_to(
                urls,
                receiver.deref(),
                rumor.deref().clone(),
                expiration.map(|t| *t),
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

    /// Negentropy reconciliation
    ///
    /// <https://github.com/hoytech/negentropy>
    pub async fn reconcile(
        &self,
        filter: &JsFilter,
        opts: &JsNegentropyOptions,
    ) -> Result<JsReconciliationOutput> {
        self.inner
            .reconcile(filter.deref().clone(), **opts)
            .await
            .map_err(into_err)
            .map(|o| o.into())
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
