// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Rest API Client

use std::collections::HashMap;

use nostr::event::builder::Error as EventBuilderError;
use nostr::key::XOnlyPublicKey;
use nostr::types::metadata::Error as MetadataError;
use nostr::url::Url;
use nostr::{ChannelId, Contact, Event, EventBuilder, EventId, Filter, Keys, Kind, Metadata, Tag};
use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use serde_json::Value;

/// [`Client`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reqwest error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// Url parse error
    #[error("impossible to parse URL: {0}")]
    Url(#[from] nostr::url::ParseError),
    /// [`EventBuilder`] error
    #[error("event builder error: {0}")]
    EventBuilder(#[from] EventBuilderError),
    /// Secp256k1 error
    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] nostr::secp256k1::Error),
    /// Hex error
    #[error("hex decoding error: {0}")]
    Hex(#[from] nostr::hashes::hex::Error),
    /// Metadata error
    #[error(transparent)]
    Metadata(#[from] MetadataError),
    /// Json error
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Message error
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Deserialize)]
struct Response<T> {
    success: bool,
    message: String,
    data: T,
}

/// Nostr client
#[derive(Debug, Clone)]
pub struct Client {
    reqwest: ReqwestClient,
    endpoint: Url,
    keys: Keys,
}

impl Client {
    /// Create a new [`Client`]
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// let my_keys = Keys::generate();
    /// let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// let client = Client::new(&my_keys, endpoint);
    /// ```
    pub fn new(keys: &Keys, endpoint: Url) -> Self {
        Self {
            reqwest: ReqwestClient::new(),
            endpoint,
            keys: keys.clone(),
        }
    }

    /// Get current [`Keys`]
    pub fn keys(&self) -> Keys {
        self.keys.clone()
    }

    /// Get endpoint
    pub fn endpoint(&self) -> Url {
        self.endpoint.clone()
    }

    /// Get events of filters
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
    /// let subscription = Filter::new()
    ///     .pubkeys(vec![my_keys.public_key()])
    ///     .since(Timestamp::now());
    ///
    /// let _events = client.get_events_of(vec![subscription]).await.unwrap();
    /// # }
    /// ```
    pub async fn get_events_of(&self, filters: Vec<Filter>) -> Result<Vec<Event>, Error> {
        let endpoint = format!("{}events", self.endpoint);
        let req = self.reqwest.post(endpoint).json(&filters);
        let res = req.send().await?;
        let response: Response<Vec<Event>> = res.json().await?;
        if response.success {
            Ok(response.data)
        } else {
            Err(Error::Message(response.message))
        }
    }

    /// Send event
    pub async fn send_event(&self, event: Event) -> Result<EventId, Error> {
        let event_id = event.id;
        let endpoint = format!("{}event", self.endpoint);
        let req = self.reqwest.post(endpoint).json(&event);
        let res = req.send().await?;
        let response: Response<Value> = res.json().await?;
        if response.success {
            Ok(event_id)
        } else {
            Err(Error::Message(response.message))
        }
    }

    /// Send event to specific relay
    pub async fn send_event_to<S>(&self, url: S, event: Event) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let url = Url::parse(&url.into())?;
        let event_id = event.id;
        let endpoint = format!("{}event", url);
        let req = self.reqwest.post(endpoint).json(&event);
        let res = req.send().await?;
        let response: Response<Value> = res.json().await?;
        if response.success {
            Ok(event_id)
        } else {
            Err(Error::Message(response.message))
        }
    }

    async fn send_event_builder(&self, builder: EventBuilder) -> Result<EventId, Error> {
        let event: Event = builder.to_event(&self.keys)?;
        self.send_event(event).await
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
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
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
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
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
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
    /// client
    ///     .publish_pow_text_note("My first POW text note from Nostr SDK!", &[], 16)
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
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
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
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
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
    /// let _list = client.get_contact_list().await.unwrap();
    /// # }
    /// ```
    pub async fn get_contact_list(&self) -> Result<Vec<Contact>, Error> {
        let mut contact_list: Vec<Contact> = Vec::new();

        let filter = Filter::new()
            .authors(vec![self.keys.public_key()])
            .kind(Kind::ContactList)
            .limit(1);
        let events: Vec<Event> = self.get_events_of(vec![filter]).await?;

        for event in events.into_iter() {
            for tag in event.tags.into_iter() {
                match tag {
                    Tag::PubKey(pk, relay_url) => {
                        contact_list.push(Contact::new(pk, relay_url, None))
                    }
                    Tag::ContactList {
                        pk,
                        relay_url,
                        alias,
                    } => contact_list.push(Contact::new(pk, relay_url, alias)),
                    _ => (),
                }
            }
        }

        Ok(contact_list)
    }

    /// Get contact list public keys
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn get_contact_list_public_keys(&self) -> Result<Vec<XOnlyPublicKey>, Error> {
        let mut pubkeys: Vec<XOnlyPublicKey> = Vec::new();

        let filter = Filter::new()
            .authors(vec![self.keys.public_key()])
            .kind(Kind::ContactList)
            .limit(1);
        let events: Vec<Event> = self.get_events_of(vec![filter]).await?;

        for event in events.into_iter() {
            for tag in event.tags.into_iter() {
                match tag {
                    Tag::PubKey(pk, _) => pubkeys.push(pk),
                    Tag::ContactList { pk, .. } => pubkeys.push(pk),
                    _ => (),
                }
            }
        }

        Ok(pubkeys)
    }

    /// Get contact list [`Metadata`]
    pub async fn get_contact_list_metadata(
        &self,
    ) -> Result<HashMap<XOnlyPublicKey, Metadata>, Error> {
        let public_keys = self.get_contact_list_public_keys().await?;
        let mut contacts: HashMap<XOnlyPublicKey, Metadata> =
            public_keys.iter().map(|p| (*p, Metadata::new())).collect();

        // TODO: let user choose the chunk size
        let chunk_size: usize = 10;
        for chunk in public_keys.chunks(chunk_size) {
            let mut filters: Vec<Filter> = Vec::new();
            for public_key in chunk.iter() {
                filters.push(
                    Filter::new()
                        .author(*public_key)
                        .kind(Kind::Metadata)
                        .limit(1),
                );
            }
            let events: Vec<Event> = self.get_events_of(filters).await?;
            for event in events.into_iter() {
                let metadata = Metadata::from_json(&event.content)?;
                if let Some(m) = contacts.get_mut(&event.pubkey) {
                    *m = metadata
                };
            }
        }

        Ok(contacts)
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
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
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
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
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
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
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
    /// #   let endpoint = Url::parse("http://127.0.0.1:7773").unwrap();
    /// #   let client = Client::new(&my_keys, endpoint);
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
    pub async fn get_channels(&self) -> Result<Vec<Event>, Error> {
        self.get_events_of(vec![Filter::new().kind(Kind::ChannelCreation)])
            .await
    }
}
