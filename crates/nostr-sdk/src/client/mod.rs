// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin_hashes::sha256::Hash;
use nostr::key::XOnlyPublicKey;
use nostr::{
    Contact, Event, EventBuilder, Keys, Kind, KindBase, Metadata, SubscriptionFilter, Tag,
};
use tokio::sync::broadcast;
use url::Url;

#[cfg(feature = "blocking")]
pub mod blocking;

use crate::relay::pool::{RelayPool, RelayPoolNotifications};

pub struct Client {
    pool: RelayPool,
    keys: Keys,
}

impl Client {
    /// Create a new `Client`
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::Client;
    ///
    /// let my_keys = Client::generate_keys();
    /// let mut client = Client::new(&my_keys);
    /// ```
    pub fn new(keys: &Keys) -> Self {
        Self {
            pool: RelayPool::new(),
            keys: keys.clone(),
        }
    }

    /// Generate new random keys using entorpy from OS
    pub fn generate_keys() -> Keys {
        Keys::generate_from_os_random()
    }

    /// Get new notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotifications> {
        self.pool.notifications()
    }

    /// Add new relay
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    /// # let my_keys = Client::generate_keys();
    /// # let mut client = Client::new(&my_keys);
    /// client.add_relay("wss://relay.nostr.info", None).unwrap();
    /// client.add_relay("wss://relay.damus.io", None).unwrap();
    /// ```
    pub fn add_relay(&mut self, url: &str, proxy: Option<SocketAddr>) -> Result<()> {
        self.pool.add_relay(url, proxy)
    }

    /// Disconnect and remove relay
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client.remove_relay("wss://relay.nostr.info").await.unwrap();
    /// # }
    /// ```
    pub async fn remove_relay(&mut self, url: &str) -> Result<()> {
        self.pool.remove_relay(url).await
    }

    /// Connect relay
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client
    ///     .connect_relay("wss://relay.nostr.info", true)
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn connect_relay(&mut self, url: &str, wait_for_connection: bool) -> Result<()> {
        if let Some(relay) = self.pool.relays().get(url) {
            return self.pool.connect_relay(relay, wait_for_connection).await;
        }

        Err(anyhow!("Relay url not found"))
    }

    /// Disconnect relay
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client
    ///     .disconnect_relay("wss://relay.nostr.info")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn disconnect_relay(&mut self, url: &str) -> Result<()> {
        if let Some(relay) = self.pool.relays().get(url) {
            return self.pool.disconnect_relay(relay).await;
        }

        Err(anyhow!("Relay url not found"))
    }

    /// Connect to all added relays without waiting for connection and keep connection alive
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client.connect().await.unwrap();
    /// # }
    /// ```
    pub async fn connect(&mut self) -> Result<()> {
        self.pool.connect(false).await
    }

    /// Connect to all added relays waiting for initial connection and keep connection alive
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client.connect_and_wait().await.unwrap();
    /// # }
    /// ```
    pub async fn connect_and_wait(&mut self) -> Result<()> {
        self.pool.connect(true).await
    }

    /// Disconnect from all relays
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client.disconnect().await.unwrap();
    /// # }
    /// ```
    pub async fn disconnect(&mut self) -> Result<()> {
        self.pool.disconnect().await
    }

    /// Subscribe to filters
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    /// use nostr::util::time;
    /// use nostr::SubscriptionFilter;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// let subscription = SubscriptionFilter::new()
    ///     .pubkeys(vec![my_keys.public_key()])
    ///     .since(time::timestamp());
    ///
    /// client.subscribe(vec![subscription]).await.unwrap();
    /// # }
    /// ```
    pub async fn subscribe(&mut self, filters: Vec<SubscriptionFilter>) -> Result<()> {
        self.pool.subscribe(filters).await
    }

    /// Get events of filters
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    /// use nostr::util::time;
    /// use nostr::SubscriptionFilter;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// let subscription = SubscriptionFilter::new()
    ///     .pubkeys(vec![my_keys.public_key()])
    ///     .since(time::timestamp());
    ///
    /// let _events = client.get_events_of(vec![subscription]).await.unwrap();
    /// # }
    /// ```
    pub async fn get_events_of(&self, filters: Vec<SubscriptionFilter>) -> Result<Vec<Event>> {
        self.pool.get_events_of(filters).await
    }

    /// Send event
    pub async fn send_event(&self, event: Event) -> Result<()> {
        self.pool.send_event(event).await
    }

    /// Update profile metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    /// use nostr_sdk::nostr::Metadata;
    /// use url::Url;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// let metadata = Metadata::new()
    ///     .name("username")
    ///     .display_name("My Username")
    ///     .about("Description")
    ///     .picture(Url::parse("https://example.com/avatar.png").unwrap())
    ///     .nip05("username@example.com");
    ///
    /// client.update_profile(metadata).await.unwrap();
    /// # }
    /// ```
    pub async fn update_profile(&self, metadata: Metadata) -> Result<()> {
        let event: Event =
            EventBuilder::set_metadata(&self.keys, metadata)?.to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Publish text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client
    ///     .publish_text_note("My first text note from Nostr SDK!", &[])
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn publish_text_note(&self, content: &str, tags: &[Tag]) -> Result<()> {
        let event: Event = EventBuilder::new_text_note(content, tags).to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Publish POW text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client
    ///     .publish_pow_text_note("My first POW text note from Nostr SDK!", &[], 16)
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn publish_pow_text_note(
        &self,
        content: &str,
        tags: &[Tag],
        difficulty: u8,
    ) -> Result<()> {
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
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// client
    ///     .add_recommended_relay("wss://relay.damus.io")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn add_recommended_relay(&self, url: &str) -> Result<()> {
        let url = Url::from_str(url)?;
        let event: Event = EventBuilder::add_recommended_relay(&url).to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Set contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn set_contact_list(&self, list: Vec<Contact>) -> Result<()> {
        let event: Event = EventBuilder::set_contact_list(list).to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Get contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// let _list = client.get_contact_list().await.unwrap();
    /// # }
    /// ```
    pub async fn get_contact_list(&mut self) -> Result<Vec<Contact>> {
        let mut contact_list: Vec<Contact> = Vec::new();

        let filter = SubscriptionFilter::new()
            .authors(vec![self.keys.public_key()])
            .kind(Kind::Base(KindBase::ContactList))
            .limit(1);
        let events: Vec<Event> = self.get_events_of(vec![filter]).await?;

        for event in events.into_iter() {
            for tag in event.tags.into_iter() {
                let tag: Vec<String> = tag.as_vec();
                if let Some(pk) = tag.get(1) {
                    let pk = XOnlyPublicKey::from_str(pk)?;
                    let relay_url = tag.get(2).cloned();
                    let alias = tag.get(3).cloned();
                    contact_list.push(Contact::new(
                        pk,
                        relay_url.unwrap_or_default(),
                        alias.unwrap_or_default(),
                    ));
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
    /// use nostr::key::{FromBech32, Keys};
    /// use nostr_sdk::Client;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    ///
    /// client.add_relay("wss://relay.nostr.info", None).unwrap();
    /// client.connect().await.unwrap();
    ///
    /// let alice_keys = Keys::from_bech32_public_key(
    ///     "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy",
    /// )
    /// .unwrap();
    ///
    /// client
    ///     .send_direct_msg(&alice_keys, "My first DM fro Nostr SDK!")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn send_direct_msg(&self, recipient: &Keys, msg: &str) -> Result<()> {
        let event: Event = EventBuilder::new_encrypted_direct_msg(&self.keys, recipient, msg)?
            .to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Delete event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    pub async fn delete_event(&self, event_id: &str) -> Result<()> {
        let event: Event =
            EventBuilder::delete(vec![Hash::from_str(event_id)?], None).to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Like event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    /// use nostr::Event;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// let event = Event::from_json(r#"{
    ///         "pubkey":"a8e76c3ace7829f9ee44cf9293309e21a1824bf1e57631d00685a1ed0b0bd8a2",
    ///         "content":"🔥 78,680 blocks to the next Halving 🔥",
    ///         "id":"3aded8d2194dc2fedb1d7b70480b43b6c4deb0a22dcdc9c471d1958485abcf21",
    ///         "created_at":1667337749,
    ///         "sig":"96e0a125e15ecc889757a1b517fdab0223a9ceae22d2591536b5f5186599b50cb1c5f20c2d0d06cdd5cd75368529e33bac4fcd3b321db9865d47785b95b72625",
    ///         "kind":1,
    ///         "tags":[]
    ///     }"#).unwrap();
    ///
    /// client.like(&event).await.unwrap();
    /// # }
    /// ```
    pub async fn like(&self, event: &Event) -> Result<()> {
        let event: Event = EventBuilder::new_reaction(event, true).to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Disike event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::Client;
    /// use nostr::Event;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Client::generate_keys();
    /// #   let mut client = Client::new(&my_keys);
    /// let event = Event::from_json(r#"{
    ///     "pubkey":"a8e76c3ace7829f9ee44cf9293309e21a1824bf1e57631d00685a1ed0b0bd8a2",
    ///     "content":"🔥 78,680 blocks to the next Halving 🔥",
    ///     "id":"3aded8d2194dc2fedb1d7b70480b43b6c4deb0a22dcdc9c471d1958485abcf21",
    ///     "created_at":1667337749,
    ///     "sig":"96e0a125e15ecc889757a1b517fdab0223a9ceae22d2591536b5f5186599b50cb1c5f20c2d0d06cdd5cd75368529e33bac4fcd3b321db9865d47785b95b72625",
    ///     "kind":1,
    ///     "tags":[]
    /// }"#).unwrap();
    ///
    /// client.dislike(&event).await.unwrap();
    /// # }
    /// ```
    pub async fn dislike(&self, event: &Event) -> Result<()> {
        let event: Event = EventBuilder::new_reaction(event, false).to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn new_channel(
        &self,
        name: &str,
        about: Option<&str>,
        picture: Option<&str>,
    ) -> Result<()> {
        let event: Event = EventBuilder::new_channel(name, about, picture)?.to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Update channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn update_channel(
        &self,
        channel_id: Hash,
        relay_url: Url,
        name: Option<&str>,
        about: Option<&str>,
        picture: Option<&str>,
    ) -> Result<()> {
        let event: Event =
            EventBuilder::set_channel_metadata(channel_id, relay_url, name, about, picture)?
                .to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Send message to channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn send_channel_msg(
        &self,
        channel_id: Hash,
        relay_url: Url,
        msg: &str,
    ) -> Result<()> {
        let event: Event =
            EventBuilder::new_channel_msg(channel_id, relay_url, msg).to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Hide channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn hide_channel_msg(&self, message_id: Hash, reason: &str) -> Result<()> {
        let event: Event =
            EventBuilder::hide_channel_msg(message_id, reason).to_event(&self.keys)?;
        self.send_event(event).await
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn mute_channel_user(&self, pubkey: XOnlyPublicKey, reason: &str) -> Result<()> {
        let event: Event = EventBuilder::mute_channel_user(pubkey, reason).to_event(&self.keys)?;
        self.send_event(event).await
    }

    pub async fn handle_notifications<F>(&self, func: F) -> Result<()>
    where
        F: Fn(RelayPoolNotifications) -> Result<()>,
    {
        loop {
            let mut notifications = self.notifications();

            while let Ok(notification) = notifications.recv().await {
                func(notification)?;
            }
        }
    }
}
