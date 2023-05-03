// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

// use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;

use js_sys::Array;
use nostr_js::error::{into_err, Result};
use nostr_js::util;
use nostr_js::{
    JsChannelId, JsContact, JsEvent, JsEventId, JsFilter, JsKeys, JsMetadata, JsPublicKey,
};
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

// use crate::relay::JsRelay;

#[wasm_bindgen(js_name = Client)]
pub struct JsClient {
    inner: Client,
}

#[wasm_bindgen(js_class = Client)]
impl JsClient {
    #[wasm_bindgen(constructor)]
    pub fn new(keys: &JsKeys) -> Self {
        Self {
            inner: Client::new(keys.deref()),
        }
    }

    /// Update default difficulty for new `Event`
    #[wasm_bindgen(js_name = updateDifficulty)]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    /// Get current `Keys`
    #[wasm_bindgen(getter)]
    pub fn keys(&self) -> JsKeys {
        self.inner.keys().into()
    }

    /// Completly shutdown `Client`
    #[wasm_bindgen]
    pub async fn shutdown(&self) -> Result<()> {
        self.inner.clone().shutdown().await.map_err(into_err)
    }

    /* /// Get relays
    #[wasm_bindgen]
    pub async fn relays(&self) -> HashMap<String, JsRelay> {
        self.inner
            .relays()
            .await
            .into_iter()
            .map(|(u, r)| (u.to_string(), r.into()))
            .collect()
    } */

    /// Add new relay
    #[wasm_bindgen(js_name = addRelay)]
    pub async fn add_relay(&self, url: String) -> Result<()> {
        self.inner.add_relay(url, None).await.map_err(into_err)
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
    #[wasm_bindgen]
    pub async fn connect(&self) {
        self.inner.connect().await;
    }

    /// Disconnect from all relays
    #[wasm_bindgen]
    pub async fn disconnect(&self) -> Result<()> {
        self.inner.disconnect().await.map_err(into_err)
    }

    /// Subscribe to filters
    #[wasm_bindgen]
    pub async fn subscribe(&self, filters: Array) -> Result<()> {
        let filters = filters
            .iter()
            .map(|v| Ok(util::downcast::<JsFilter>(&v, "Filter")?.inner()))
            .collect::<Result<Vec<Filter>, JsError>>()?;
        self.inner.subscribe(filters).await;
        Ok(())
    }

    /// Unsubscribe
    #[wasm_bindgen]
    pub async fn unsubscribe(&self) {
        self.inner.unsubscribe().await;
    }

    /// Get events of filters
    #[wasm_bindgen(js_name = getEventsOf)]
    pub async fn get_events_of(&self, filters: Array, timeout: Option<u64>) -> Result<Array> {
        let filters = filters
            .iter()
            .map(|v| Ok(util::downcast::<JsFilter>(&v, "Filter")?.inner()))
            .collect::<Result<Vec<Filter>, JsError>>()?;
        let timeout = timeout.map(Duration::from_secs);
        match self.inner.get_events_of(filters, timeout).await {
            Ok(events) => {
                let events: Vec<JsEvent> = events.into_iter().map(|e| e.into()).collect();
                let events = events.into_iter().map(JsValue::from).collect();
                Ok(events)
            }
            Err(e) => Err(e).map_err(into_err),
        }
    }

    /// Request events of filters.
    /// All events will be received on notification listener
    /// until the EOSE "end of stored events" message is received from the relay.
    #[wasm_bindgen(js_name = reqEventsOf)]
    pub async fn req_events_of(&self, filters: Array, timeout: Option<u64>) -> Result<()> {
        let filters = filters
            .iter()
            .map(|v| Ok(util::downcast::<JsFilter>(&v, "Filter")?.inner()))
            .collect::<Result<Vec<Filter>, JsError>>()?;
        let timeout = timeout.map(Duration::from_secs);
        self.inner.req_events_of(filters, timeout).await;
        Ok(())
    }

    /// Send event
    #[wasm_bindgen(js_name = sendEvent)]
    pub async fn send_event(&self, event: JsEvent) -> Result<JsEventId> {
        self.inner
            .send_event(event.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send event to specific relay
    #[wasm_bindgen(js_name = sendEventTo)]
    pub async fn send_event_to(&self, url: String, event: JsEvent) -> Result<JsEventId> {
        self.inner
            .send_event_to(url, event.into())
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
            .set_metadata(metadata.into())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Publish text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = publishTextNote)]
    pub async fn publish_text_note(&self, content: String, tags: Array) -> Result<JsEventId> {
        let tags: Vec<Vec<String>> = serde_wasm_bindgen::from_value(tags.into())?;
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

    /// Add recommended relay
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = addRecommendedRelay)]
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
    #[wasm_bindgen(js_name = setContactList)]
    pub async fn set_contact_list(&self, list: Array) -> Result<JsEventId> {
        let list = list
            .iter()
            .map(|v| Ok(util::downcast::<JsContact>(&v, "Contact")?.inner()))
            .collect::<Result<Vec<Contact>, JsError>>()?;
        self.inner
            .set_contact_list(list)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /* /// Get contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[wasm_bindgen(js_name = getContactList)]
    pub async fn get_contact_list(&self, timeout: Option<u64>) -> Result<Vec<JsContact>> {
        let timeout = timeout.map(|t| Duration::from_secs(t as u64));
        self.inner
            .get_contact_list(timeout)
            .await
            .map_err(into_err)
            .map(|vec| vec.into_iter().map(|c| c.into()).collect())
    }

    /// Get contact list public keys
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[wasm_bindgen(js_name = getContactListPublicKeys)]
    pub async fn get_contact_list_public_keys(
        &self,
        timeout: Option<u64>,
    ) -> Result<Vec<JsPublicKey>> {
        let timeout = timeout.map(|t| Duration::from_secs(t as u64));
        self.inner
            .get_contact_list_public_keys(timeout)
            .await
            .map_err(into_err)
            .map(|vec| vec.into_iter().map(|c| c.into()).collect())
    } */

    /* /// Get contact list [`Metadata`]
    #[wasm_bindgen(js_name = getContactListMetadata)]
    pub async fn get_contact_list_metadata(
        &self,
        timeout: Option<u64>,
    ) -> Result<HashMap<JsPublicKey, JsMetadata>> {
        let timeout = timeout.map(|t| Duration::from_secs(t as u64));
        self.inner
            .get_contact_list_public_keys(timeout)
            .await
            .map_err(into_err)
            .map(|vec| vec.into_iter().map(|c| c.into()).collect())
    } */

    /// Send encrypted direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/04.md>
    #[wasm_bindgen(js_name = sendDirectMsg)]
    pub async fn send_direct_msg(&self, receiver: &JsPublicKey, msg: String) -> Result<JsEventId> {
        self.inner
            .send_direct_msg(receiver.into(), msg)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Repost event
    #[wasm_bindgen(js_name = repostEvent)]
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
    #[wasm_bindgen(js_name = deleteEvent)]
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
    #[wasm_bindgen]
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
    #[wasm_bindgen]
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
    #[wasm_bindgen]
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
    #[wasm_bindgen(js_name = newChannel)]
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
    #[wasm_bindgen(js_name = setChannelMetadata)]
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
    #[wasm_bindgen(js_name = sendChannelMsg)]
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
    #[wasm_bindgen(js_name = hideChannelUser)]
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
            .mute_channel_user(pubkey.into(), reason)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }
}
