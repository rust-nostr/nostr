// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(non_snake_case)]

// use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;

use async_utility::thread;
use js_sys::Array;
use nostr_js::error::{into_err, Result};
use nostr_js::event::{JsEvent, JsEventArray, JsEventBuilder, JsEventId, JsTag};
use nostr_js::key::JsPublicKey;
use nostr_js::message::{JsFilter, JsRelayMessage};
use nostr_js::types::{JsContact, JsMetadata};
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod builder;
pub mod signer;

pub use self::signer::JsClientSigner;
use crate::abortable::JsAbortHandle;
use crate::database::JsNostrDatabase;

// use crate::relay::JsRelay;

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
    pub fn new(signer: Option<JsClientSigner>) -> Self {
        Self {
            inner: match signer {
                Some(signer) => Client::new(signer.deref().clone()),
                None => ClientBuilder::new().build(),
            },
        }
    }

    /// Update default difficulty for new `Event`
    #[wasm_bindgen(js_name = updateDifficulty)]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.inner.update_difficulty(difficulty);
    }

    /// Get current client signer
    ///
    /// Rise error if it not set.
    #[wasm_bindgen(getter)]
    pub async fn signer(&self) -> Result<JsClientSigner> {
        Ok(self.inner.signer().await.map_err(into_err)?.into())
    }

    #[wasm_bindgen(getter)]
    pub fn database(&self) -> JsNostrDatabase {
        self.inner.database().into()
    }

    /// Completely shutdown `Client`
    #[wasm_bindgen]
    pub async fn shutdown(self) -> Result<()> {
        self.inner.shutdown().await.map_err(into_err)
    }

    // /// Get relays
    // #[wasm_bindgen]
    // pub async fn relays(&self) -> HashMap<String, JsRelay> {
    // self.inner
    // .relays()
    // .await
    // .into_iter()
    // .map(|(u, r)| (u.to_string(), r.into()))
    // .collect()
    // }

    /// Add new relay
    #[wasm_bindgen(js_name = addRelay)]
    pub async fn add_relay(&self, url: String) -> Result<bool> {
        self.inner.add_relay(url).await.map_err(into_err)
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
    pub async fn subscribe(&self, filters: Vec<JsFilter>) -> Result<()> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.inner()).collect();
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
    pub async fn get_events_of(
        &self,
        filters: Vec<JsFilter>,
        timeout: Option<f64>,
    ) -> Result<JsEventArray> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.inner()).collect();
        let timeout: Option<Duration> = timeout.map(Duration::from_secs_f64);
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

    /// Request events of filters.
    /// All events will be received on notification listener
    /// until the EOSE "end of stored events" message is received from the relay.
    #[wasm_bindgen(js_name = reqEventsOf)]
    pub async fn req_events_of(&self, filters: Vec<JsFilter>, timeout: Option<f64>) -> Result<()> {
        let filters: Vec<Filter> = filters.into_iter().map(|f| f.inner()).collect();
        let timeout = timeout.map(Duration::from_secs_f64);
        self.inner.req_events_of(filters, timeout).await;
        Ok(())
    }

    /// Send event
    #[wasm_bindgen(js_name = sendEvent)]
    pub async fn send_event(&self, event: &JsEvent) -> Result<JsEventId> {
        self.inner
            .send_event(event.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Send event to specific relay
    #[wasm_bindgen(js_name = sendEventTo)]
    pub async fn send_event_to(&self, url: String, event: &JsEvent) -> Result<JsEventId> {
        self.inner
            .send_event_to(url, event.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Take an [`EventBuilder`], sign it by using the [`ClientSigner`] and broadcast to all relays.
    ///
    /// Rise an error if the [`ClientSigner`] is not set.
    #[wasm_bindgen(js_name = sendEventBuilder)]
    pub async fn send_event_builder(&self, builder: &JsEventBuilder) -> Result<JsEventId> {
        self.inner
            .send_event_builder(builder.deref().clone())
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    /// Take an [`EventBuilder`], sign it by using the [`ClientSigner`] and broadcast to specific relays.
    ///
    /// Rise an error if the [`ClientSigner`] is not set.
    #[wasm_bindgen(js_name = sendEventBuilderTo)]
    pub async fn send_event_builder_to(
        &self,
        url: String,
        builder: &JsEventBuilder,
    ) -> Result<JsEventId> {
        self.inner
            .send_event_builder_to(url, builder.deref().clone())
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
    pub async fn set_contact_list(&self, list: Vec<JsContact>) -> Result<JsEventId> {
        let list = list.into_iter().map(|c| c.inner());
        self.inner
            .set_contact_list(list)
            .await
            .map_err(into_err)
            .map(|id| id.into())
    }

    // /// Get contact list
    //
    // <https://github.com/nostr-protocol/nips/blob/master/02.md>
    // #[wasm_bindgen(js_name = getContactList)]
    // pub async fn get_contact_list(&self, timeout: Option<u64>) -> Result<Vec<JsContact>> {
    // let timeout = timeout.map(|t| Duration::from_secs(t as u64));
    // self.inner
    // .get_contact_list(timeout)
    // .await
    // .map_err(into_err)
    // .map(|vec| vec.into_iter().map(|c| c.into()).collect())
    // }
    //
    // Get contact list public keys
    //
    // <https://github.com/nostr-protocol/nips/blob/master/02.md>
    // #[wasm_bindgen(js_name = getContactListPublicKeys)]
    // pub async fn get_contact_list_public_keys(
    // &self,
    // timeout: Option<u64>,
    // ) -> Result<Vec<JsPublicKey>> {
    // let timeout = timeout.map(|t| Duration::from_secs(t as u64));
    // self.inner
    // .get_contact_list_public_keys(timeout)
    // .await
    // .map_err(into_err)
    // .map(|vec| vec.into_iter().map(|c| c.into()).collect())
    // }

    // /// Get contact list [`Metadata`]
    // #[wasm_bindgen(js_name = getContactListMetadata)]
    // pub async fn get_contact_list_metadata(
    // &self,
    // timeout: Option<u64>,
    // ) -> Result<HashMap<JsPublicKey, JsMetadata>> {
    // let timeout = timeout.map(|t| Duration::from_secs(t as u64));
    // self.inner
    // .get_contact_list_public_keys(timeout)
    // .await
    // .map_err(into_err)
    // .map(|vec| vec.into_iter().map(|c| c.into()).collect())
    // }

    /// Send encrypted direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/04.md>
    #[wasm_bindgen(js_name = sendDirectMsg)]
    pub async fn send_direct_msg(
        &self,
        receiver: &JsPublicKey,
        msg: String,
        reply: Option<JsEventId>,
    ) -> Result<JsEventId> {
        self.inner
            .send_direct_msg(receiver.into(), msg, reply.map(|id| id.into()))
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
            .set_channel_metadata(channel_id.into(), relay_url, metadata.deref())
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

    /// Handle notifications
    ///
    /// **This method spawn a thread**, so ensure to keep up the app after calling this (if needed).
    ///
    /// To exit from the handle notifications loop, return `true`, the
    ///
    /// # Example
    /// ```javascript
    /// // Subscribe to filters
    /// const filter = new Filter().author(keys.publicKey);
    /// await client.subscribe([filter]);
    ///
    /// const handle = {
    ///    // Handle event
    ///    handleEvent: async (relayUrl, event) => {
    ///        console.log("Received new event from", relayUrl);
    ///        if (event.kind == 4) {
    ///            try {
    ///                let content = nip04_decrypt(keys.secretKey, event.pubkey, event.content);
    ///                console.log("Message:", content);
    ///                await client.sendDirectMsg(event.pubkey, "Echo: " + content);
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
    pub fn handle_notifications(&self, callback: HandleNotification) -> JsAbortHandle {
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
                    RelayPoolNotification::Event { relay_url, event } => {
                        let event: JsEvent = event.into();
                        if callback.handle_event(relay_url.to_string(), event).await.as_bool().unwrap_or_default() {
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
        handle.into()
    }
}

#[wasm_bindgen(typescript_custom_section)]
const HANDLE_NOTIFICATION: &'static str = r#"
interface HandleNotification {
    handleEvent: (relayUrl: string, event: Event) => Promise<boolean>;
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
        event: JsEvent,
    ) -> JsValue;

    #[wasm_bindgen(structural, method, js_name = handleMsg)]
    pub async fn handle_msg(
        this: &HandleNotification,
        relay_url: String,
        message: JsRelayMessage,
    ) -> JsValue;
}
