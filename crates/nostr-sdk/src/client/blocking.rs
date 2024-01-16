// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use nostr::key::XOnlyPublicKey;
use nostr::nips::nip94::FileMetadata;
use nostr::url::Url;
use nostr::{
    ClientMessage, Contact, Event, EventBuilder, EventId, Filter, Keys, Metadata, Result, Tag,
};
use nostr_database::DynNostrDatabase;
use tokio::sync::broadcast;

use super::signer::ClientSigner;
use super::{Error, Options, TryIntoUrl};
use crate::relay::{pool, Relay, RelayOptions, RelayPoolNotification};
use crate::{ClientBuilder, NegentropyOptions, RUNTIME};

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) client: super::Client,
}

impl From<super::Client> for Client {
    fn from(client: super::Client) -> Self {
        Self { client }
    }
}

impl Client {
    pub fn new<S>(signer: S) -> Self
    where
        S: Into<ClientSigner>,
    {
        Self {
            client: super::Client::new(signer),
        }
    }

    pub fn with_opts<S>(signer: S, opts: Options) -> Self
    where
        S: Into<ClientSigner>,
    {
        Self {
            client: super::Client::with_opts(signer, opts),
        }
    }

    /// Compose [`Client`] from [`ClientBuilder`]
    pub fn from_builder(builder: ClientBuilder) -> Self {
        Self {
            client: super::Client::from_builder(builder),
        }
    }

    pub fn update_difficulty(&self, difficulty: u8) {
        self.client.update_difficulty(difficulty);
    }

    /// Get current client signer
    ///
    /// Rise error if it not set.
    pub fn signer(&self) -> Result<ClientSigner, Error> {
        RUNTIME.block_on(async { self.client.signer().await })
    }

    /// Set client signer
    pub fn set_signer(&self, signer: Option<ClientSigner>) {
        RUNTIME.block_on(async { self.client.set_signer(signer).await })
    }

    /// Get current [`Keys`]
    #[deprecated(since = "0.27.0", note = "Use `client.signer()` instead.")]
    pub fn keys(&self) -> Keys {
        #[allow(deprecated)]
        RUNTIME.block_on(async { self.client.keys().await })
    }

    /// Change [`Keys`]
    #[deprecated(since = "0.27.0", note = "Use `client.set_signer(...)` instead.")]
    pub fn set_keys(&self, keys: &Keys) {
        #[allow(deprecated)]
        RUNTIME.block_on(async { self.client.set_keys(keys).await })
    }

    /// Get database
    pub fn database(&self) -> Arc<DynNostrDatabase> {
        self.client.database()
    }

    /// Start a previously stopped client
    pub fn start(&self) {
        RUNTIME.block_on(async { self.client.start().await })
    }

    /// Stop the client
    pub fn stop(&self) -> Result<(), Error> {
        RUNTIME.block_on(async { self.client.stop().await })
    }

    /// Check if [`RelayPool`](super::RelayPool) is running
    pub fn is_running(&self) -> bool {
        self.client.is_running()
    }

    /// Completely shutdown [`Client`]
    pub fn shutdown(self) -> Result<(), Error> {
        RUNTIME.block_on(async { self.client.shutdown().await })
    }

    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.client.notifications()
    }

    /// Get relays
    pub fn relays(&self) -> HashMap<Url, Relay> {
        RUNTIME.block_on(async { self.client.relays().await })
    }

    /// Get [`Relay`]
    pub fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.relay(url).await })
    }

    /// Add multiple relays
    pub fn add_relays<I, U>(&self, relays: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.add_relays(relays).await })
    }

    pub fn add_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.add_relay(url).await })
    }

    pub fn add_relay_with_opts<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.add_relay_with_opts(url, opts).await })
    }

    pub fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.remove_relay(url).await })
    }

    pub fn connect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.connect_relay(url).await })
    }

    pub fn disconnect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.disconnect_relay(url).await })
    }

    pub fn connect(&self) {
        RUNTIME.block_on(async {
            self.client.connect().await;
        })
    }

    pub fn disconnect(&self) -> Result<(), Error> {
        RUNTIME.block_on(async { self.client.disconnect().await })
    }

    pub fn subscribe(&self, filters: Vec<Filter>) {
        RUNTIME.block_on(async {
            self.client.subscribe(filters).await;
        })
    }

    pub fn unsubscribe(&self) {
        RUNTIME.block_on(async {
            self.client.unsubscribe().await;
        })
    }

    pub fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        RUNTIME.block_on(async { self.client.get_events_of(filters, timeout).await })
    }

    pub fn req_events_of(&self, filters: Vec<Filter>, timeout: Option<Duration>) {
        RUNTIME.block_on(async {
            self.client.req_events_of(filters, timeout).await;
        })
    }

    pub fn send_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        RUNTIME.block_on(async { self.client.send_msg(msg).await })
    }

    pub fn send_msg_to<U>(&self, url: U, msg: ClientMessage) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.send_msg_to(url, msg).await })
    }

    /// Send event
    pub fn send_event(&self, event: Event) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.send_event(event).await })
    }

    pub fn send_event_to<U>(&self, url: U, event: Event) -> Result<EventId, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.send_event_to(url, event).await })
    }

    pub fn send_event_builder(&self, builder: EventBuilder) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.send_event_builder(builder).await })
    }

    pub fn send_event_builder_to<U>(&self, url: U, builder: EventBuilder) -> Result<EventId, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        RUNTIME.block_on(async { self.client.send_event_builder_to(url, builder).await })
    }

    pub fn set_metadata(&self, metadata: &Metadata) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.set_metadata(metadata).await })
    }

    pub fn publish_text_note<S, I>(&self, content: S, tags: I) -> Result<EventId, Error>
    where
        S: Into<String>,
        I: IntoIterator<Item = Tag>,
    {
        RUNTIME.block_on(async { self.client.publish_text_note(content, tags).await })
    }

    #[deprecated(since = "0.27.0")]
    pub fn add_recommended_relay<U>(&self, url: U) -> Result<EventId, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        #[allow(deprecated)]
        RUNTIME.block_on(async { self.client.add_recommended_relay(url).await })
    }

    pub fn set_contact_list(&self, list: Vec<Contact>) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.set_contact_list(list).await })
    }

    pub fn get_contact_list(&self, timeout: Option<Duration>) -> Result<Vec<Contact>, Error> {
        RUNTIME.block_on(async { self.client.get_contact_list(timeout).await })
    }

    pub fn get_contact_list_public_keys(
        &self,
        timeout: Option<Duration>,
    ) -> Result<Vec<XOnlyPublicKey>, Error> {
        RUNTIME.block_on(async { self.client.get_contact_list_public_keys(timeout).await })
    }

    pub fn get_contact_list_metadata(
        &self,
        timeout: Option<Duration>,
    ) -> Result<HashMap<XOnlyPublicKey, Metadata>, Error> {
        RUNTIME.block_on(async { self.client.get_contact_list_metadata(timeout).await })
    }

    #[cfg(feature = "nip04")]
    pub fn send_direct_msg<S>(
        &self,
        receiver: XOnlyPublicKey,
        msg: S,
        reply: Option<EventId>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        RUNTIME.block_on(async { self.client.send_direct_msg(receiver, msg, reply).await })
    }

    pub fn repost_event(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
    ) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.repost_event(event_id, public_key).await })
    }

    pub fn delete_event(&self, event_id: EventId) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.delete_event(event_id).await })
    }

    pub fn like(&self, event_id: EventId, public_key: XOnlyPublicKey) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.like(event_id, public_key).await })
    }

    pub fn dislike(&self, event_id: EventId, public_key: XOnlyPublicKey) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.dislike(event_id, public_key).await })
    }

    pub fn reaction<S>(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
        content: S,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        RUNTIME.block_on(async { self.client.reaction(event_id, public_key, content).await })
    }

    pub fn new_channel(&self, metadata: &Metadata) -> Result<EventId, Error> {
        RUNTIME.block_on(async { self.client.new_channel(metadata).await })
    }

    pub fn set_channel_metadata(
        &self,
        channel_id: EventId,
        relay_url: Option<Url>,
        metadata: &Metadata,
    ) -> Result<EventId, Error> {
        RUNTIME.block_on(async {
            self.client
                .set_channel_metadata(channel_id, relay_url, metadata)
                .await
        })
    }

    pub fn send_channel_msg<S>(
        &self,
        channel_id: EventId,
        relay_url: Url,
        msg: S,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        RUNTIME.block_on(async {
            self.client
                .send_channel_msg(channel_id, relay_url, msg)
                .await
        })
    }

    pub fn hide_channel_msg<S>(
        &self,
        message_id: EventId,
        reason: Option<S>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        RUNTIME.block_on(async { self.client.hide_channel_msg(message_id, reason).await })
    }

    pub fn mute_channel_user<S>(
        &self,
        pubkey: XOnlyPublicKey,
        reason: Option<S>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        RUNTIME.block_on(async { self.client.mute_channel_user(pubkey, reason).await })
    }

    /// Create an auth event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    pub fn auth<S>(&self, challenge: S, relay: Url) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        RUNTIME.block_on(async { self.client.auth(challenge, relay).await })
    }

    /// Create zap receipt event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[cfg(feature = "nip57")]
    pub fn zap_receipt<S>(
        &self,
        bolt11: S,
        preimage: Option<S>,
        zap_request: Event,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        RUNTIME.block_on(async { self.client.zap_receipt(bolt11, preimage, zap_request).await })
    }

    /// Create zap receipt event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[cfg(feature = "nip57")]
    #[deprecated(since = "0.27.0", note = "Use `zap_receipt` instead")]
    pub fn new_zap_receipt<S>(
        &self,
        bolt11: S,
        preimage: Option<S>,
        zap_request: Event,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        self.zap_receipt(bolt11, preimage, zap_request)
    }

    /// File metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/94.md>
    pub fn file_metadata<S>(&self, description: S, metadata: FileMetadata) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        RUNTIME.block_on(async { self.client.file_metadata(description, metadata).await })
    }

    /// Negentropy reconciliation
    pub fn reconcile(&self, filter: Filter, opts: NegentropyOptions) -> Result<(), Error> {
        RUNTIME.block_on(async move { self.client.reconcile(filter, opts).await })
    }

    #[deprecated(since = "0.27.0")]
    pub fn get_channels(&self, timeout: Option<Duration>) -> Result<Vec<Event>, Error> {
        #[allow(deprecated)]
        RUNTIME.block_on(async { self.client.get_channels(timeout).await })
    }

    pub fn handle_notifications<F>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(RelayPoolNotification) -> Result<bool>,
    {
        let mut notifications = self.client.notifications();
        while let Ok(notification) = RUNTIME.block_on(notifications.recv()) {
            let stop: bool = RelayPoolNotification::Stop == notification;
            let shutdown: bool = RelayPoolNotification::Shutdown == notification;
            let exit: bool = func(notification).map_err(|e| Error::Handler(e.to_string()))?;
            if exit || stop || shutdown {
                break;
            }
        }
        Ok(())
    }
}
