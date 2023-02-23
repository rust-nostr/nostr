// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Client

use std::collections::HashMap;
use std::net::SocketAddr;
#[cfg(feature = "sqlite")]
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use nostr::event::builder::Error as EventBuilderError;
use nostr::key::XOnlyPublicKey;
use nostr::url::Url;
use nostr::{
    ChannelId, ClientMessage, Contact, Entity, Event, EventBuilder, EventId, Filter, Keys, Kind,
    Metadata, Tag,
};
#[cfg(feature = "sqlite")]
use nostr_sdk_sqlite::Store;
use tokio::sync::broadcast;

#[cfg(feature = "blocking")]
pub mod blocking;
mod options;

pub use self::options::Options;
use crate::relay::pool::{Error as RelayPoolError, RelayPool, RelayPoolNotification};
use crate::{Relay, RelayOptions};

/// [`Client`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Url parse error
    #[error("impossible to parse URL: {0}")]
    Url(#[from] nostr::url::ParseError),
    /// [`RelayPool`] error
    #[error("relay pool error: {0}")]
    RelayPool(#[from] RelayPoolError),
    /// Relay not found
    #[error("relay not found")]
    RelayNotFound,
    /// [`EventBuilder`] error
    #[error("event builder error: {0}")]
    EventBuilder(#[from] EventBuilderError),
    /// Secp256k1 error
    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] nostr::secp256k1::Error),
    /// Hex error
    #[error("hex decoding error: {0}")]
    Hex(#[from] nostr::hashes::hex::Error),
}

/// Nostr client
#[derive(Debug, Clone)]
pub struct Client {
    pool: RelayPool,
    keys: Keys,
    opts: Options,
}

impl Client {
    /// Create a new [`Client`]
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// let my_keys = Keys::generate();
    /// let client = Client::new(&my_keys);
    /// ```
    pub fn new(keys: &Keys) -> Self {
        Self::new_with_opts(keys, Options::default())
    }

    /// Create a new [`Client`] with [`Options`]
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// let my_keys = Keys::generate();
    /// let opts = Options::new().wait_for_send(true);
    /// let client = Client::new_with_opts(&my_keys, opts);
    /// ```
    pub fn new_with_opts(keys: &Keys, opts: Options) -> Self {
        Self {
            pool: RelayPool::new(),
            keys: keys.clone(),
            opts,
        }
    }

    /// New [`Client`] with [`Store`]
    #[cfg(feature = "sqlite")]
    pub fn new_with_store<P>(keys: &Keys, path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Self::new_with_store_and_opts(keys, path, Options::default())
    }

    /// New [`Client`] with [`Store`] and [`Options`]
    #[cfg(feature = "sqlite")]
    pub fn new_with_store_and_opts<P>(keys: &Keys, path: P, opts: Options) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            pool: RelayPool::new_with_store(path)?,
            keys: keys.clone(),
            opts,
        })
    }

    /// Update default difficulty for new [`Event`]
    #[cfg(feature = "nip13")]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.opts.update_difficulty(difficulty);
    }

    /// Enable/Disable Nostr Connect (NIP46)
    #[cfg(feature = "nip46")]
    pub fn nostr_connect(&self, enable: bool) {
        self.opts.update_nostr_connect(enable);
    }

    /// Get current [`Keys`]
    pub fn keys(&self) -> Keys {
        self.keys.clone()
    }

    /// Get [`Store`]
    #[cfg(feature = "sqlite")]
    pub fn store(&self) -> Option<Store> {
        self.pool.store()
    }

    /// Completly shutdown [`Client`]
    pub async fn shutdown(self) -> Result<(), Error> {
        Ok(self.pool.shutdown().await?)
    }

    /// Get new notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.pool.notifications()
    }

    /// Get relays
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        self.pool.relays().await
    }

    /// Add new relay
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .add_relay("wss://relay.nostr.info", None)
    ///     .await
    ///     .unwrap();
    /// client
    ///     .add_relay("wss://relay.damus.io", None)
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn add_relay<S>(&self, url: S, proxy: Option<SocketAddr>) -> Result<(), Error>
    where
        S: Into<String>,
    {
        self.add_relay_with_opts(url, proxy, RelayOptions::default())
            .await
    }

    /// Add new relay
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let read = true;
    /// let write = false;
    /// let opts = RelayOptions::new(read, write);
    /// client
    ///     .add_relay_with_opts("wss://relay.nostr.info", None, opts)
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn add_relay_with_opts<S>(
        &self,
        url: S,
        proxy: Option<SocketAddr>,
        opts: RelayOptions,
    ) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let url = Url::parse(&url.into())?;
        self.pool.add_relay(url, proxy, opts).await?;
        Ok(())
    }

    /// Disconnect and remove relay
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client.remove_relay("wss://relay.nostr.info").await.unwrap();
    /// # }
    /// ```
    pub async fn remove_relay<S>(&self, url: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let url = Url::parse(&url.into())?;
        self.pool.remove_relay(url).await?;
        Ok(())
    }

    /// Add multiple relays
    pub async fn add_relays<S>(&self, relays: Vec<(S, Option<SocketAddr>)>) -> Result<(), Error>
    where
        S: Into<String>,
    {
        for (url, proxy) in relays.into_iter() {
            self.add_relay(url, proxy).await?;
        }
        Ok(())
    }

    /// Restore previous added relays from store
    #[cfg(feature = "sqlite")]
    pub async fn restore_relays(&self) -> Result<(), Error> {
        Ok(self.pool.restore_relays().await?)
    }

    /// Connect relay
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .connect_relay("wss://relay.nostr.info")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn connect_relay<S>(&self, url: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let url = Url::parse(&url.into())?;
        if let Some(relay) = self.pool.relays().await.get(&url) {
            self.pool
                .connect_relay(relay, self.opts.get_wait_for_connection())
                .await;
            return Ok(());
        }
        Err(Error::RelayNotFound)
    }

    /// Disconnect relay
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .disconnect_relay("wss://relay.nostr.info")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn disconnect_relay<S>(&self, url: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let url = Url::parse(&url.into())?;
        if let Some(relay) = self.pool.relays().await.get(&url) {
            self.pool.disconnect_relay(relay).await?;
            return Ok(());
        }
        Err(Error::RelayNotFound)
    }

    /// Connect to all added relays
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client.connect().await;
    /// # }
    /// ```
    pub async fn connect(&self) {
        self.pool.connect(self.opts.get_wait_for_connection()).await;
    }

    /// Disconnect from all relays
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client.disconnect().await.unwrap();
    /// # }
    /// ```
    pub async fn disconnect(&self) -> Result<(), Error> {
        Ok(self.pool.disconnect().await?)
    }

    /// Subscribe to filters
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let subscription = Filter::new()
    ///     .pubkeys(vec![my_keys.public_key()])
    ///     .since(Timestamp::now());
    ///
    /// client.subscribe(vec![subscription]).await;
    /// # }
    /// ```
    pub async fn subscribe(&self, filters: Vec<Filter>) {
        self.pool
            .subscribe(filters, self.opts.get_wait_for_send())
            .await;
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self) {
        self.pool.unsubscribe(self.opts.get_wait_for_send()).await;
    }

    /// Get events of filters
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::time::Duration;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let subscription = Filter::new()
    ///     .pubkeys(vec![my_keys.public_key()])
    ///     .since(Timestamp::now());
    ///
    /// let timeout = Duration::from_secs(10);
    /// let _events = client
    ///     .get_events_of(vec![subscription], Some(timeout))
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        Ok(self.pool.get_events_of(filters, timeout).await?)
    }

    /// Request events of filters
    /// All events will be received on notification listener (`client.notifications()`)
    pub async fn req_events_of(&self, filters: Vec<Filter>, timeout: Option<Duration>) {
        self.pool.req_events_of(filters, timeout).await;
    }

    /// Send client message
    pub async fn send_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        Ok(self
            .pool
            .send_msg(msg, self.opts.get_wait_for_send())
            .await?)
    }

    /// Send client message to a specific relay
    pub async fn send_msg_to<S>(&self, url: S, msg: ClientMessage) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let url = Url::parse(&url.into())?;
        Ok(self
            .pool
            .send_msg_to(url, msg, self.opts.get_wait_for_send())
            .await?)
    }

    /// Send event
    pub async fn send_event(&self, event: Event) -> Result<EventId, Error> {
        let event_id = event.id;
        self.send_msg(ClientMessage::new_event(event)).await?;
        Ok(event_id)
    }

    /// Send event to specific relay
    pub async fn send_event_to<S>(&self, url: S, event: Event) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let event_id = event.id;
        self.send_msg_to(url, ClientMessage::new_event(event))
            .await?;
        Ok(event_id)
    }

    async fn send_event_builder(&self, builder: EventBuilder) -> Result<EventId, Error> {
        #[cfg(feature = "nip13")]
        let event: Event = {
            let difficulty: u8 = self.opts.get_difficulty();
            if difficulty > 0 {
                builder.to_pow_event(&self.keys, difficulty)?
            } else {
                builder.to_event(&self.keys)?
            }
        };
        #[cfg(not(feature = "nip13"))]
        let event: Event = builder.to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Update profile metadata
    #[deprecated(since = "0.19.0", note = "Use `set_metadata` method")]
    pub async fn update_profile(&self, metadata: Metadata) -> Result<EventId, Error> {
        self.set_metadata(metadata).await
    }

    /// Update metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let metadata = Metadata::new()
    ///     .name("username")
    ///     .display_name("My Username")
    ///     .about("Description")
    ///     .picture(Url::parse("https://example.com/avatar.png").unwrap())
    ///     .nip05("username@example.com");
    ///
    /// client.set_metadata(metadata).await.unwrap();
    /// # }
    /// ```
    pub async fn set_metadata(&self, metadata: Metadata) -> Result<EventId, Error> {
        let builder = EventBuilder::set_metadata(metadata);
        self.send_event_builder(builder).await
    }

    /// Publish text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .publish_text_note("My first text note from Nostr SDK!", &[])
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn publish_text_note<S>(&self, content: S, tags: &[Tag]) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::new_text_note(content, tags);
        self.send_event_builder(builder).await
    }

    /// Publish POW text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .publish_pow_text_note("My first POW text note from Nostr SDK!", &[], 16)
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    #[cfg(feature = "nip13")]
    pub async fn publish_pow_text_note<S>(
        &self,
        content: S,
        tags: &[Tag],
        difficulty: u8,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let event: Event =
            EventBuilder::new_text_note(content, tags).to_pow_event(&self.keys, difficulty)?;
        self.send_event(event).await
    }

    /// Add recommended relay
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .add_recommended_relay("wss://relay.damus.io")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn add_recommended_relay<S>(&self, url: S) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let url = Url::parse(&url.into())?;
        let builder = EventBuilder::add_recommended_relay(&url);
        self.send_event_builder(builder).await
    }

    /// Set contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn set_contact_list(&self, list: Vec<Contact>) -> Result<EventId, Error> {
        let builder = EventBuilder::set_contact_list(list);
        self.send_event_builder(builder).await
    }

    /// Get contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::time::Duration;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let timeout = Duration::from_secs(10);
    /// let _list = client.get_contact_list(Some(timeout)).await.unwrap();
    /// # }
    /// ```
    pub async fn get_contact_list(&self, timeout: Option<Duration>) -> Result<Vec<Contact>, Error> {
        let mut contact_list: Vec<Contact> = Vec::new();

        let filter = Filter::new()
            .authors(vec![self.keys.public_key()])
            .kind(Kind::ContactList)
            .limit(1);
        let events: Vec<Event> = self.get_events_of(vec![filter], timeout).await?;

        for event in events.into_iter() {
            for tag in event.tags.into_iter() {
                if let Tag::ContactList {
                    pk,
                    relay_url,
                    alias,
                } = tag
                {
                    contact_list.push(Contact::new(pk, relay_url, alias));
                }
            }
        }

        Ok(contact_list)
    }

    /// Send encrypted direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/04.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let alice_pubkey = XOnlyPublicKey::from_bech32(
    ///     "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy",
    /// )
    /// .unwrap();
    ///
    /// client
    ///     .send_direct_msg(alice_pubkey, "My first DM fro Nostr SDK!")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    #[cfg(feature = "nip04")]
    pub async fn send_direct_msg<S>(
        &self,
        receiver: XOnlyPublicKey,
        msg: S,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::new_encrypted_direct_msg(&self.keys, receiver, msg)?;
        self.send_event_builder(builder).await
    }

    /// Repost event
    pub async fn repost_event(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::repost(event_id, public_key);
        self.send_event_builder(builder).await
    }

    /// Delete event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    pub async fn delete_event<S>(
        &self,
        event_id: EventId,
        reason: Option<S>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::delete(vec![event_id], reason);
        self.send_event_builder(builder).await
    }

    /// Like event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::str::FromStr;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let event_id =
    ///     EventId::from_hex("3aded8d2194dc2fedb1d7b70480b43b6c4deb0a22dcdc9c471d1958485abcf21")
    ///         .unwrap();
    /// let public_key = XOnlyPublicKey::from_str(
    ///     "a8e76c3ace7829f9ee44cf9293309e21a1824bf1e57631d00685a1ed0b0bd8a2",
    /// )
    /// .unwrap();
    ///
    /// client.like(event_id, public_key).await.unwrap();
    /// # }
    /// ```
    pub async fn like(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::new_reaction(event_id, public_key, "+");
        self.send_event_builder(builder).await
    }

    /// Disike event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::str::FromStr;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let event_id =
    ///     EventId::from_hex("3aded8d2194dc2fedb1d7b70480b43b6c4deb0a22dcdc9c471d1958485abcf21")
    ///         .unwrap();
    /// let public_key = XOnlyPublicKey::from_str(
    ///     "a8e76c3ace7829f9ee44cf9293309e21a1824bf1e57631d00685a1ed0b0bd8a2",
    /// )
    /// .unwrap();
    ///
    /// client.dislike(event_id, public_key).await.unwrap();
    /// # }
    /// ```
    pub async fn dislike(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::new_reaction(event_id, public_key, "-");
        self.send_event_builder(builder).await
    }

    /// React to an [`Event`]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::str::FromStr;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let event_id =
    ///     EventId::from_hex("3aded8d2194dc2fedb1d7b70480b43b6c4deb0a22dcdc9c471d1958485abcf21")
    ///         .unwrap();
    /// let public_key = XOnlyPublicKey::from_str(
    ///     "a8e76c3ace7829f9ee44cf9293309e21a1824bf1e57631d00685a1ed0b0bd8a2",
    /// )
    /// .unwrap();
    ///
    /// client.reaction(event_id, public_key, "🐻").await.unwrap();
    /// # }
    /// ```
    pub async fn reaction<S>(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
        content: S,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::new_reaction(event_id, public_key, content);
        self.send_event_builder(builder).await
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn new_channel(&self, metadata: Metadata) -> Result<EventId, Error> {
        let builder = EventBuilder::new_channel(metadata);
        self.send_event_builder(builder).await
    }

    /// Update channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[deprecated(since = "0.19.0", note = "Use `set_channel_metadata` method")]
    pub async fn update_channel(
        &self,
        channel_id: ChannelId,
        relay_url: Option<Url>,
        metadata: Metadata,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::set_channel_metadata(channel_id, relay_url, metadata);
        self.send_event_builder(builder).await
    }

    /// Update channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn set_channel_metadata(
        &self,
        channel_id: ChannelId,
        relay_url: Option<Url>,
        metadata: Metadata,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::set_channel_metadata(channel_id, relay_url, metadata);
        self.send_event_builder(builder).await
    }

    /// Send message to channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn send_channel_msg<S>(
        &self,
        channel_id: ChannelId,
        relay_url: Url,
        msg: S,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::new_channel_msg(channel_id, relay_url, msg);
        self.send_event_builder(builder).await
    }

    /// Hide channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn hide_channel_msg<S>(
        &self,
        message_id: EventId,
        reason: Option<S>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::hide_channel_msg(message_id, reason);
        self.send_event_builder(builder).await
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn mute_channel_user<S>(
        &self,
        pubkey: XOnlyPublicKey,
        reason: Option<S>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::mute_channel_user(pubkey, reason);
        self.send_event_builder(builder).await
    }

    /// Get a list of channels
    pub async fn get_channels(&self, timeout: Option<Duration>) -> Result<Vec<Event>, Error> {
        self.get_events_of(vec![Filter::new().kind(Kind::ChannelCreation)], timeout)
            .await
    }

    /// Get entity of hex string
    pub async fn get_entity_of<S>(
        &self,
        entity: S,
        timeout: Option<Duration>,
    ) -> Result<Entity, Error>
    where
        S: Into<String>,
    {
        let entity: String = entity.into();
        let events: Vec<Event> = self
            .get_events_of(
                vec![Filter::new()
                    .id(&entity)
                    .kind(Kind::ChannelCreation)
                    .limit(1)],
                timeout,
            )
            .await?;
        if events.is_empty() {
            let pubkey = XOnlyPublicKey::from_str(&entity)?;
            let events: Vec<Event> = self
                .get_events_of(vec![Filter::new().author(pubkey).limit(1)], timeout)
                .await?;
            if events.is_empty() {
                Ok(Entity::Unknown)
            } else {
                Ok(Entity::Account)
            }
        } else {
            Ok(Entity::Channel)
        }
    }

    /// Handle notifications
    pub async fn handle_notifications<F>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(RelayPoolNotification) -> Result<(), Error>,
    {
        loop {
            let mut notifications = self.notifications();

            while let Ok(notification) = notifications.recv().await {
                func(notification)?;
            }
        }
    }
}
