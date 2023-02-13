// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::time::Duration;

use napi::Result;
use nostr_nodejs::{
    JsChannelId, JsContact, JsEvent, JsEventId, JsKeys, JsMetadata, JsPublicKey,
    JsSubscriptionFilter,
};
use nostr_sdk::prelude::*;

mod options;

pub use self::options::JsOptions;
use crate::error::into_err;
use crate::relay::JsRelay;

#[napi(js_name = "Client")]
pub struct JsClient {
    inner: Client,
}

#[napi]
impl JsClient {
    #[napi(constructor)]
    pub fn new(keys: &JsKeys) -> Self {
        Self {
            inner: Client::new(keys.deref()),
        }
    }

    /// Create a new `Client` with custom `Options`
    #[napi(factory)]
    pub fn new_with_opts(keys: &JsKeys, opts: &JsOptions) -> Self {
        Self {
            inner: Client::new_with_opts(keys.deref(), opts.into()),
        }
    }

    /// Update default difficulty for new `Event`
    #[napi]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    /// Get current `Keys`
    #[napi]
    pub fn keys(&self) -> JsKeys {
        self.inner.keys().into()
    }

    /// Completly shutdown `Client`
    #[napi]
    pub async fn shutdown(&self) -> Result<()> {
        self.inner.clone().shutdown().await.map_err(into_err)
    }

    // Add notifications

    /// Get relays
    #[napi]
    pub async fn relays(&self) -> HashMap<String, JsRelay> {
        self.inner
            .relays()
            .await
            .into_iter()
            .map(|(u, r)| (u.to_string(), r.into()))
            .collect()
    }

    /// Add new relay
    #[napi]
    pub async fn add_relay(&self, url: String, proxy: Option<String>) -> Result<()> {
        let proxy: Option<SocketAddr> = match proxy {
            Some(proxy) => Some(proxy.parse().map_err(into_err)?),
            None => None,
        };
        self.inner.add_relay(url, proxy).await.map_err(into_err)
    }

    // Add relay with opts

    /// Remove relay
    #[napi]
    pub async fn remove_relay(&self, url: String) -> Result<()> {
        self.inner.remove_relay(url).await.map_err(into_err)
    }

    /// Connect relay
    #[napi]
    pub async fn connect_relay(&self, url: String) -> Result<()> {
        self.inner.connect_relay(url).await.map_err(into_err)
    }

    /// Disconnect relay
    #[napi]
    pub async fn disconnect_relay(&self, url: String) -> Result<()> {
        self.inner.disconnect_relay(url).await.map_err(into_err)
    }

    /// Connect to all added relays
    #[napi]
    pub async fn connect(&self) {
        self.inner.connect().await;
    }

    /// Disconnect from all relays
    #[napi]
    pub async fn disconnect(&self) -> Result<()> {
        self.inner.disconnect().await.map_err(into_err)
    }

    /// Subscribe to filters
    #[napi]
    pub async fn subscribe(&self, filters: Vec<&JsSubscriptionFilter>) {
        let filters = filters.into_iter().map(|f| f.into()).collect();
        self.inner.subscribe(filters).await;
    }

    /// Unsubscribe
    #[napi]
    pub async fn unsubscribe(&self) {
        self.inner.unsubscribe().await;
    }

    /// Get events of filters
    #[napi]
    pub async fn get_events_of(
        &self,
        filters: Vec<&JsSubscriptionFilter>,
        timeout: Option<u32>,
    ) -> Result<Vec<JsEvent>> {
        let filters = filters.into_iter().map(|f| f.into()).collect();
        let timeout = timeout.map(|t| Duration::from_secs(t as u64));
        match self.inner.get_events_of(filters, timeout).await {
            Ok(events) => {
                let events = events.into_iter().map(|e| e.into()).collect();
                Ok(events)
            }
            Err(e) => Err(e).map_err(into_err),
        }
    }

    /// Request events of filters
    /// All events will be received on notification listener
    #[napi]
    pub async fn req_events_of(&self, filters: Vec<&JsSubscriptionFilter>, timeout: Option<u32>) {
        let filters = filters.into_iter().map(|f| f.into()).collect();
        let timeout = timeout.map(|t| Duration::from_secs(t as u64));
        self.inner.req_events_of(filters, timeout).await;
    }

    // Add Send client message

    // Add Send client message to a specific relay

    /// Send event
    #[napi]
    pub async fn send_event(&self, event: &JsEvent) -> Result<JsEventId> {
        self.inner
            .send_event(event.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send event to specific relay
    #[napi]
    pub async fn send_event_to(&self, url: String, event: &JsEvent) -> Result<JsEventId> {
        self.inner
            .send_event_to(url, event.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Update metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[napi]
    pub async fn set_metadata(&self, metadata: &JsMetadata) -> Result<JsEventId> {
        self.inner
            .set_metadata(metadata.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Publish text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[napi]
    pub async fn publish_text_note(
        &self,
        content: String,
        tags: Vec<Vec<String>>,
    ) -> Result<JsEventId> {
        let mut new_tags: Vec<Tag> = Vec::with_capacity(tags.len());
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }
        self.inner
            .publish_text_note(content, &new_tags)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Publish POW text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    #[napi]
    pub async fn publish_pow_text_note(
        &self,
        content: String,
        tags: Vec<Vec<String>>,
        difficulty: u8,
    ) -> Result<JsEventId> {
        let mut new_tags: Vec<Tag> = Vec::with_capacity(tags.len());
        for tag in tags.into_iter() {
            new_tags.push(Tag::try_from(tag).map_err(into_err)?);
        }
        self.inner
            .publish_pow_text_note(content, &new_tags, difficulty)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Add recommended relay
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[napi]
    pub async fn add_recommended_relay(&self, url: String) -> Result<JsEventId> {
        self.inner
            .add_recommended_relay(url)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Set contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[napi]
    pub async fn set_contact_list(&self, list: Vec<&JsContact>) -> Result<JsEventId> {
        let list = list.into_iter().map(|c| c.into()).collect();
        self.inner
            .set_contact_list(list)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Get contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[napi]
    pub async fn get_contact_list(&self, timeout: Option<u32>) -> Result<Vec<JsContact>> {
        let timeout = timeout.map(|t| Duration::from_secs(t as u64));
        self.inner
            .get_contact_list(timeout)
            .await
            .map_err(into_err)
            .map(|vec| vec.into_iter().map(|c| c.into()).collect())
    }

    /// Send encrypted direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/04.md>
    #[napi]
    pub async fn send_direct_msg(&self, receiver: &JsPublicKey, msg: String) -> Result<JsEventId> {
        self.inner
            .send_direct_msg(receiver.into(), msg)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Repost event
    #[napi]
    pub async fn repost_event(
        &self,
        event_id: &JsEventId,
        public_key: &JsPublicKey,
    ) -> Result<JsEventId> {
        self.inner
            .repost_event(event_id.into(), public_key.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Delete event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    #[napi]
    pub async fn delete_event(
        &self,
        event_id: &JsEventId,
        reason: Option<String>,
    ) -> Result<JsEventId> {
        self.inner
            .delete_event(event_id.into(), reason)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Like event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    #[napi]
    pub async fn like(&self, event_id: &JsEventId, public_key: &JsPublicKey) -> Result<JsEventId> {
        self.inner
            .like(event_id.into(), public_key.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Disike event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    #[napi]
    pub async fn dislike(
        &self,
        event_id: &JsEventId,
        public_key: &JsPublicKey,
    ) -> Result<JsEventId> {
        self.inner
            .dislike(event_id.into(), public_key.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// React to an [`Event`]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    #[napi]
    pub async fn reaction(
        &self,
        event_id: &JsEventId,
        public_key: &JsPublicKey,
        content: String,
    ) -> Result<JsEventId> {
        self.inner
            .reaction(event_id.into(), public_key.into(), content)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[napi]
    pub async fn new_channel(&self, metadata: &JsMetadata) -> Result<JsEventId> {
        self.inner
            .new_channel(metadata.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Update channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[napi]
    pub async fn set_channel_metadata(
        &self,
        channel_id: &JsChannelId,
        relay_url: Option<String>,
        metadata: &JsMetadata,
    ) -> Result<JsEventId> {
        let relay_url: Option<Url> = match relay_url {
            Some(relay_url) => Some(Url::parse(&relay_url).map_err(into_err)?),
            None => None,
        };
        self.inner
            .set_channel_metadata(channel_id.into(), relay_url, metadata.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send message to channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[napi]
    pub async fn send_channel_msg(
        &self,
        channel_id: &JsChannelId,
        relay_url: String,
        msg: String,
    ) -> Result<JsEventId> {
        let relay_url: Url = Url::parse(&relay_url).map_err(into_err)?;
        self.inner
            .send_channel_msg(channel_id.into(), relay_url, msg)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Hide channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[napi]
    pub async fn hide_channel_msg(
        &self,
        message_id: &JsEventId,
        reason: Option<String>,
    ) -> Result<JsEventId> {
        self.inner
            .hide_channel_msg(message_id.into(), reason)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }
    #[napi]
    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[napi]
    pub async fn mute_channel_user(
        &self,
        pubkey: &JsPublicKey,
        reason: Option<String>,
    ) -> Result<JsEventId> {
        self.inner
            .mute_channel_user(pubkey.into(), reason)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Get a list of channels
    #[napi]
    pub async fn get_channels(&self, timeout: Option<u32>) -> Result<Vec<JsEvent>> {
        let timeout = timeout.map(|t| Duration::from_secs(t as u64));
        self.inner
            .get_channels(timeout)
            .await
            .map_err(into_err)
            .map(|vec| vec.into_iter().map(|e| e.into()).collect())
    }
}
