// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr gossip

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![allow(unknown_lints)] // TODO: remove when MSRV >= 1.72.0, required for `clippy::arc_with_non_send_sync`
#![allow(clippy::arc_with_non_send_sync)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::num::NonZeroUsize;
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use std::path::Path;
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use std::thread;

use async_utility::task;
use lru::LruCache;
use nostr::prelude::*;
use rusqlite::{Connection, OpenFlags};
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
use tokio::sync::mpsc;

mod constant;
pub mod prelude;
mod store;

use self::constant::{CACHE_SIZE, CHECK_OUTDATED_INTERVAL, PUBKEY_METADATA_OUTDATED_AFTER};
pub use self::store::error::Error;
use self::store::Store;

const P_TAG: SingleLetterTag = SingleLetterTag::lowercase(Alphabet::P);
const SQLITE_IN_MEMORY_URI: &str = "file:memdb?mode=memory&cache=shared";

/// Broken-down filters
#[derive(Debug)]
pub enum BrokenDownFilters {
    /// Filters by url
    Filters(HashMap<RelayUrl, Filter>),
    /// Filters that match a certain pattern but where no relays are available
    Orphan(Filter),
    /// Filters that can be sent to read relays (generic query, not related to public keys)
    Other(Filter),
}

#[derive(Debug, Clone)]
struct Pool {
    /// Store
    store: Arc<Mutex<Store>>,
    /// Event ingester
    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    tx: mpsc::Sender<Event>,
    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    tx: mpsc::UnboundedSender<Event>,
}

impl Pool {
    fn new(s1: Store, s2: Store) -> Self {
        // Create new asynchronous channel
        #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
        let (tx, rx) = mpsc::channel();
        #[cfg(all(target_family = "wasm", target_os = "unknown"))]
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn ingester with the store and the channel receiver
        Self::spawn_ingester(s1, rx);

        Self {
            store: Arc::new(Mutex::new(s2)),
            tx,
        }
    }

    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Store) -> R + Send + 'static,
        R: Send + 'static,
    {
        let store: Arc<Mutex<Store>> = self.store.clone();
        Ok(task::spawn_blocking(move || {
            let mut conn = store.lock().expect("Failed to lock store");
            f(&mut conn)
        })
        .await?)
    }

    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    async fn interact<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut Store) -> R + Send + 'static,
        R: Send + 'static,
    {
        let store: Arc<Mutex<Store>> = self.store.clone();
        Ok(task::spawn(async move {
            let mut conn = store.lock().expect("Failed to lock store");
            f(&mut conn)
        })
        .join()
        .await?)
    }

    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    fn spawn_ingester(mut store: Store, rx: mpsc::Receiver<Event>) {
        thread::spawn(move || {
            #[cfg(debug_assertions)]
            tracing::debug!("Gossip ingester thread started");

            let size: NonZeroUsize = CACHE_SIZE.unwrap();
            let mut cache: LruCache<EventId, ()> = LruCache::new(size);

            // Listen for items
            while let Ok(event) = rx.recv() {
                // Update cache and check if was already processed
                if cache.put(event.id, ()).is_none() {
                    // Process event
                    if let Err(e) = store.process_event(&event) {
                        tracing::error!(error = %e, "Gossip event ingestion failed.");
                    }
                }
            }

            #[cfg(debug_assertions)]
            tracing::debug!("Gossip ingester thread exited");
        });
    }

    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    fn spawn_ingester(mut store: Store, mut rx: mpsc::UnboundedReceiver<Event>) {
        task::spawn(async move {
            #[cfg(debug_assertions)]
            tracing::debug!("Gossip ingester task started");

            let size: NonZeroUsize = CACHE_SIZE.unwrap();
            let mut cache: LruCache<EventId, ()> = LruCache::new(size);

            // Listen for items
            while let Some(event) = rx.recv().await {
                // Update cache and check if was already processed
                if cache.put(event.id, ()).is_none() {
                    // Process event
                    if let Err(e) = store.process_event(&event) {
                        tracing::error!(error = %e, "Gossip event ingestion failed.");
                    }
                }
            }

            #[cfg(debug_assertions)]
            tracing::debug!("Gossip ingester task exited");
        });
    }
}

/// Gossip tracker
#[derive(Debug, Clone)]
pub struct Gossip {
    pool: Pool,
}

impl Gossip {
    /// New in-memory gossip storage
    pub fn in_memory() -> Self {
        let s1: Connection = Connection::open_with_flags(
            SQLITE_IN_MEMORY_URI,
            OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .expect("Failed to open in-memory database");
        let s1: Store = Store::new(s1);

        s1.migrate().expect("Failed to run migrations");

        let s2: Connection = Connection::open_with_flags(
            SQLITE_IN_MEMORY_URI,
            OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .expect("Failed to open in-memory database");
        let s2: Store = Store::new(s2);

        Self {
            pool: Pool::new(s1, s2),
        }
    }

    /// New persistent gossip storage
    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    pub async fn persistent<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path> + Send + 'static,
    {
        let (s1, s2) = task::spawn_blocking(move || {
            let path = path.as_ref();

            let s1: Connection = Connection::open(path)?;
            let s1: Store = Store::new(s1);

            s1.migrate()?;

            let s2: Connection = Connection::open(path)?;
            let s2: Store = Store::new(s2);

            Ok::<(Store, Store), Error>((s1, s2))
        })
        .await??;

        Ok(Self {
            pool: Pool::new(s1, s2),
        })
    }

    /// Process an [`Event`]
    ///
    /// Only the first [`MAX_RELAYS_LIST`] relays will be used for [`Kind::RelayList`] and [`Kind::InboxRelays`] lists.
    pub fn process_event(&self, event: Cow<Event>) {
        // Check if the event can be processed
        if event.kind != Kind::RelayList && event.kind != Kind::InboxRelays {
            return;
        }

        // Send to event ingester
        // An event clone may occur here
        let _ = self.pool.tx.send(event.into_owned());
    }

    /// Check for what public keys the metadata are outdated or not existent (both for NIP17 and NIP65)
    pub async fn check_outdated(
        &self,
        public_keys: BTreeSet<PublicKey>,
        kinds: Vec<Kind>,
    ) -> Result<HashSet<PublicKey>, Error> {
        self.pool
            .interact(move |store| {
                let now: Timestamp = Timestamp::now();

                let mut outdated: HashSet<PublicKey> = HashSet::new();

                for public_key in public_keys.into_iter() {
                    for kind in kinds.iter().copied() {
                        let timestamps = store.get_timestamps(&public_key, kind)?;

                        if timestamps.last_check + CHECK_OUTDATED_INTERVAL > now {
                            continue;
                        }

                        // Check if expired
                        if timestamps.created_at + PUBKEY_METADATA_OUTDATED_AFTER < now {
                            outdated.insert(public_key);
                        }
                    }
                }

                Ok(outdated)
            })
            .await?
    }

    /// Update last check
    pub async fn update_last_check(
        &self,
        public_keys: HashSet<PublicKey>,
        kinds: Vec<Kind>,
    ) -> Result<(), Error> {
        self.pool
            .interact(move |store| {
                let now: Timestamp = Timestamp::now();

                for public_key in public_keys.into_iter() {
                    store.update_last_check(&public_key, &kinds, &now)?;
                }

                Ok(())
            })
            .await?
    }

    fn get_nip17_relays(
        store: &mut Store,
        public_keys: &BTreeSet<PublicKey>,
    ) -> Result<HashSet<RelayUrl>, Error> {
        let mut urls: HashSet<RelayUrl> = HashSet::new();

        for public_key in public_keys.iter() {
            if let Some((pkid, listid)) =
                store.get_pkid_and_listid(public_key, Kind::InboxRelays)?
            {
                let relays = store.get_relays_url(pkid, listid)?;

                urls.extend(relays);
            }
        }

        Ok(urls)
    }

    fn get_nip65_relays(
        store: &mut Store,
        public_keys: &BTreeSet<PublicKey>,
        metadata: Option<RelayMetadata>,
    ) -> Result<HashSet<RelayUrl>, Error> {
        let mut urls: HashSet<RelayUrl> = HashSet::new();

        for public_key in public_keys.iter() {
            if let Some((pkid, listid)) = store.get_pkid_and_listid(public_key, Kind::RelayList)? {
                let relays = match metadata {
                    Some(metadata) => {
                        store.get_nip65_relays_url_by_metadata(pkid, listid, metadata)?
                    }
                    None => store.get_relays_url(pkid, listid)?,
                };

                urls.extend(relays);
            }
        }

        Ok(urls)
    }

    fn map_nip17_relays(
        store: &mut Store,
        public_keys: &BTreeSet<PublicKey>,
    ) -> Result<HashMap<RelayUrl, BTreeSet<PublicKey>>, Error> {
        let mut urls: HashMap<RelayUrl, BTreeSet<PublicKey>> = HashMap::new();

        for public_key in public_keys.iter() {
            if let Some((pkid, listid)) =
                store.get_pkid_and_listid(public_key, Kind::InboxRelays)?
            {
                let relays = store.get_relays_url(pkid, listid)?;

                for url in relays.into_iter() {
                    urls.entry(url)
                        .and_modify(|s| {
                            s.insert(*public_key);
                        })
                        .or_default()
                        .insert(*public_key);
                }
            }
        }

        Ok(urls)
    }

    fn map_nip65_relays(
        store: &mut Store,
        public_keys: &BTreeSet<PublicKey>,
        metadata: RelayMetadata,
    ) -> Result<HashMap<RelayUrl, BTreeSet<PublicKey>>, Error> {
        let mut urls: HashMap<RelayUrl, BTreeSet<PublicKey>> = HashMap::new();

        for public_key in public_keys.iter() {
            if let Some((pkid, listid)) = store.get_pkid_and_listid(public_key, Kind::RelayList)? {
                let relays = store.get_nip65_relays_url_by_metadata(pkid, listid, metadata)?;

                for url in relays.into_iter() {
                    urls.entry(url)
                        .and_modify(|s| {
                            s.insert(*public_key);
                        })
                        .or_default()
                        .insert(*public_key);
                }
            }
        }

        Ok(urls)
    }

    /// Get outbox (write) relays for public keys
    #[inline]
    pub async fn get_nip65_outbox_relays(
        &self,
        public_keys: BTreeSet<PublicKey>,
    ) -> Result<HashSet<RelayUrl>, Error> {
        self.pool
            .interact(move |store| {
                Self::get_nip65_relays(store, &public_keys, Some(RelayMetadata::Write))
            })
            .await?
    }

    /// Get inbox (read) relays for public keys
    #[inline]
    pub async fn get_nip65_inbox_relays(
        &self,
        public_keys: BTreeSet<PublicKey>,
    ) -> Result<HashSet<RelayUrl>, Error> {
        self.pool
            .interact(move |store| {
                Self::get_nip65_relays(store, &public_keys, Some(RelayMetadata::Read))
            })
            .await?
    }

    /// Get NIP17 inbox (read) relays for public keys
    #[inline]
    pub async fn get_nip17_inbox_relays(
        &self,
        public_keys: BTreeSet<PublicKey>,
    ) -> Result<HashSet<RelayUrl>, Error> {
        self.pool
            .interact(move |store| Self::get_nip17_relays(store, &public_keys))
            .await?
    }

    /// Map outbox (write) relays for public keys
    #[inline]
    fn map_nip65_outbox_relays(
        store: &mut Store,
        public_keys: &BTreeSet<PublicKey>,
    ) -> Result<HashMap<RelayUrl, BTreeSet<PublicKey>>, Error> {
        Self::map_nip65_relays(store, public_keys, RelayMetadata::Write)
    }

    /// Map NIP65 inbox (read) relays for public keys
    #[inline]
    fn map_nip65_inbox_relays(
        store: &mut Store,
        public_keys: &BTreeSet<PublicKey>,
    ) -> Result<HashMap<RelayUrl, BTreeSet<PublicKey>>, Error> {
        Self::map_nip65_relays(store, public_keys, RelayMetadata::Read)
    }

    /// Get NIP65 **outbox** + NIP17 relays
    async fn map_outbox_relays(
        &self,
        public_keys: BTreeSet<PublicKey>,
    ) -> Result<HashMap<RelayUrl, BTreeSet<PublicKey>>, Error> {
        self.pool
            .interact(move |store| {
                // Get map of outbox relays
                let mut relays = Self::map_nip65_outbox_relays(store, &public_keys)?;

                // Extend with NIP17 relays
                let nip17 = Self::map_nip17_relays(store, &public_keys)?;
                relays.extend(nip17);

                Ok::<_, Error>(relays)
            })
            .await?
    }

    /// Get NIP65 **inbox** + NIP17 relays
    async fn map_inbox_relays(
        &self,
        public_keys: BTreeSet<PublicKey>,
    ) -> Result<HashMap<RelayUrl, BTreeSet<PublicKey>>, Error> {
        self.pool
            .interact(move |store| {
                // Get map of inbox relays
                let mut relays = Self::map_nip65_inbox_relays(store, &public_keys)?;

                // Extend with NIP17 relays
                let nip17 = Self::map_nip17_relays(store, &public_keys)?;
                relays.extend(nip17);

                Ok::<_, Error>(relays)
            })
            .await?
    }

    /// Get NIP65 + NIP17 relays
    async fn get_relays(
        &self,
        public_keys: BTreeSet<PublicKey>,
    ) -> Result<HashSet<RelayUrl>, Error> {
        self.pool
            .interact(move |store| {
                // Get map of outbox and inbox relays
                let mut relays: HashSet<RelayUrl> =
                    Self::get_nip65_relays(store, &public_keys, None)?;

                // Extend with NIP17 relays
                let nip17 = Self::get_nip17_relays(store, &public_keys)?;
                relays.extend(nip17);

                Ok::<_, Error>(relays)
            })
            .await?
    }

    /// Break down filters
    ///
    /// The additional relays will always be used
    pub async fn break_down_filter<I>(
        &self,
        filter: Filter,
        additional_relays: I,
    ) -> Result<BrokenDownFilters, Error>
    where
        I: IntoIterator<Item = RelayUrl>,
    {
        // Extract `p` tag from generic tags and parse public key hex
        let p_tag: Option<BTreeSet<PublicKey>> = filter.generic_tags.get(&P_TAG).map(|s| {
            s.iter()
                .filter_map(|p| PublicKey::from_hex(p).ok())
                .collect()
        });

        // Match pattern
        match (filter.authors.as_ref().cloned(), p_tag) {
            (Some(authors), None) => {
                let additional_relays: HashMap<RelayUrl, BTreeSet<PublicKey>> = additional_relays
                    .into_iter()
                    .map(|r| (r, authors.clone()))
                    .collect();

                let mut outbox: HashMap<RelayUrl, BTreeSet<PublicKey>> =
                    self.map_outbox_relays(authors).await?;

                // Extend with additional relays
                outbox.extend(additional_relays);

                // No relay available for the authors
                if outbox.is_empty() {
                    return Ok(BrokenDownFilters::Orphan(filter));
                }

                let mut map: HashMap<RelayUrl, Filter> = HashMap::with_capacity(outbox.len());

                // Construct new filters
                for (relay, pk_set) in outbox.into_iter() {
                    // Clone filter and change authors
                    let mut new_filter: Filter = filter.clone();
                    new_filter.authors = Some(pk_set);

                    // Update map
                    map.insert(relay, new_filter);
                }

                Ok(BrokenDownFilters::Filters(map))
            }
            (None, Some(p_public_keys)) => {
                let additional_relays: HashMap<RelayUrl, BTreeSet<PublicKey>> = additional_relays
                    .into_iter()
                    .map(|r| (r, p_public_keys.clone()))
                    .collect();

                let mut inbox: HashMap<RelayUrl, BTreeSet<PublicKey>> =
                    self.map_inbox_relays(p_public_keys).await?;

                // Extend with additional relays
                inbox.extend(additional_relays);

                // No relay available for the p tags
                if inbox.is_empty() {
                    return Ok(BrokenDownFilters::Orphan(filter));
                }

                let mut map: HashMap<RelayUrl, Filter> = HashMap::with_capacity(inbox.len());

                // Construct new filters
                for (relay, pk_set) in inbox.into_iter() {
                    // Clone filter and change p tags
                    let mut new_filter: Filter = filter.clone();
                    new_filter
                        .generic_tags
                        .insert(P_TAG, pk_set.into_iter().map(|p| p.to_string()).collect());

                    // Update map
                    map.insert(relay, new_filter);
                }

                Ok(BrokenDownFilters::Filters(map))
            }
            (Some(authors), Some(p_public_keys)) => {
                let mut union: BTreeSet<PublicKey> = authors;
                union.extend(p_public_keys);

                let mut relays: HashSet<RelayUrl> = self.get_relays(union).await?;

                // Extend with additional relays
                relays.extend(additional_relays);

                // No relay available for the authors and p tags
                if relays.is_empty() {
                    return Ok(BrokenDownFilters::Orphan(filter));
                }

                let mut map: HashMap<RelayUrl, Filter> = HashMap::with_capacity(relays.len());

                for relay in relays.into_iter() {
                    // Update map
                    map.insert(relay, filter.clone());
                }

                Ok(BrokenDownFilters::Filters(map))
            }
            // Nothing to do, add to `other` list
            (None, None) => Ok(BrokenDownFilters::Other(filter)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    const SECRET_KEY_A: &str = "nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99"; // aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4
    const SECRET_KEY_B: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85"; // 79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3

    const KEY_A_RELAYS: [(&str, Option<RelayMetadata>); 4] = [
        ("wss://relay.damus.io", None),
        ("wss://relay.nostr.bg", None),
        ("wss://nos.lol", Some(RelayMetadata::Write)),
        ("wss://nostr.mom", Some(RelayMetadata::Read)),
    ];

    const KEY_B_RELAYS: [(&str, Option<RelayMetadata>); 4] = [
        ("wss://relay.damus.io", Some(RelayMetadata::Write)),
        ("wss://relay.nostr.info", None),
        ("wss://relay.rip", Some(RelayMetadata::Write)),
        ("wss://relay.snort.social", Some(RelayMetadata::Read)),
    ];

    fn build_relay_list_event(
        secret_key: &str,
        relays: Vec<(&str, Option<RelayMetadata>)>,
    ) -> Event {
        let keys = Keys::parse(secret_key).unwrap();
        let list = relays
            .into_iter()
            .filter_map(|(url, m)| Some((RelayUrl::parse(url).ok()?, m)));
        EventBuilder::relay_list(list)
            .sign_with_keys(&keys)
            .unwrap()
    }

    async fn setup_graph() -> Gossip {
        let graph = Gossip::in_memory();

        let events = vec![
            build_relay_list_event(SECRET_KEY_A, KEY_A_RELAYS.to_vec()),
            build_relay_list_event(SECRET_KEY_B, KEY_B_RELAYS.to_vec()),
        ];

        for event in events {
            graph.process_event(Cow::Owned(event));
        }

        // Wait to allow to process events
        tokio::time::sleep(Duration::from_secs(1)).await;

        graph
    }

    #[tokio::test]
    async fn test_break_down_filter() {
        let keys_a = Keys::parse(SECRET_KEY_A).unwrap();
        let keys_b = Keys::parse(SECRET_KEY_B).unwrap();

        let damus_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let nostr_bg_url = RelayUrl::parse("wss://relay.nostr.bg").unwrap();
        let nos_lol_url = RelayUrl::parse("wss://nos.lol").unwrap();
        let nostr_mom_url = RelayUrl::parse("wss://nostr.mom").unwrap();
        let nostr_info_url = RelayUrl::parse("wss://relay.nostr.info").unwrap();
        let relay_rip_url = RelayUrl::parse("wss://relay.rip").unwrap();
        let snort_url = RelayUrl::parse("wss://relay.snort.social").unwrap();

        let graph = setup_graph().await;

        // Single author
        let filter = Filter::new().author(keys_a.public_key);
        match graph
            .break_down_filter(filter.clone(), HashSet::new())
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &filter);
                assert_eq!(map.get(&nostr_bg_url).unwrap(), &filter);
                assert_eq!(map.get(&nos_lol_url).unwrap(), &filter);
                assert!(!map.contains_key(&nostr_mom_url));
            }
            _ => panic!("Expected filters"),
        }

        // Single author with additional relays
        let filter = Filter::new().author(keys_a.public_key);
        match graph
            .break_down_filter(filter.clone(), HashSet::new())
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &filter);
                assert_eq!(map.get(&nostr_bg_url).unwrap(), &filter);
                assert_eq!(map.get(&nos_lol_url).unwrap(), &filter);
                assert!(!map.contains_key(&nostr_mom_url));
            }
            _ => panic!("Expected filters"),
        }

        // Multiple authors
        let additional_relay = RelayUrl::parse("wss://relay.example.com").unwrap();
        let authors_filter = Filter::new().authors([keys_a.public_key, keys_b.public_key]);
        match graph
            .break_down_filter(
                authors_filter.clone(),
                HashSet::from([additional_relay.clone()]),
            )
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &authors_filter);
                assert_eq!(
                    map.get(&nostr_bg_url).unwrap(),
                    &Filter::new().author(keys_a.public_key)
                );
                assert_eq!(
                    map.get(&nos_lol_url).unwrap(),
                    &Filter::new().author(keys_a.public_key)
                );
                assert!(!map.contains_key(&nostr_mom_url));
                assert_eq!(
                    map.get(&nostr_info_url).unwrap(),
                    &Filter::new().author(keys_b.public_key)
                );
                assert_eq!(
                    map.get(&relay_rip_url).unwrap(),
                    &Filter::new().author(keys_b.public_key)
                );
                assert_eq!(map.get(&additional_relay).unwrap(), &authors_filter);
                assert!(!map.contains_key(&snort_url));
            }
            _ => panic!("Expected filters"),
        }

        // Other filter
        let search_filter = Filter::new().search("Test").limit(10);
        match graph
            .break_down_filter(search_filter.clone(), HashSet::new())
            .await
            .unwrap()
        {
            BrokenDownFilters::Other(filter) => {
                assert_eq!(filter, search_filter);
            }
            _ => panic!("Expected other"),
        }

        // Single p tags
        let p_tag_filter = Filter::new().pubkey(keys_a.public_key);
        match graph
            .break_down_filter(p_tag_filter.clone(), HashSet::new())
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &p_tag_filter);
                assert_eq!(map.get(&nostr_bg_url).unwrap(), &p_tag_filter);
                assert_eq!(map.get(&nostr_mom_url).unwrap(), &p_tag_filter);
                assert!(!map.contains_key(&nos_lol_url));
                assert!(!map.contains_key(&nostr_info_url));
                assert!(!map.contains_key(&relay_rip_url));
                assert!(!map.contains_key(&snort_url));
            }
            _ => panic!("Expected filters"),
        }

        // Both author and p tag
        let filter = Filter::new()
            .author(keys_a.public_key)
            .pubkey(keys_b.public_key);
        match graph
            .break_down_filter(filter.clone(), HashSet::new())
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &filter);
                assert_eq!(map.get(&nostr_bg_url).unwrap(), &filter);
                assert_eq!(map.get(&nos_lol_url).unwrap(), &filter);
                assert_eq!(map.get(&nostr_mom_url).unwrap(), &filter);
                assert_eq!(map.get(&nostr_info_url).unwrap(), &filter);
                assert_eq!(map.get(&relay_rip_url).unwrap(), &filter);
                assert_eq!(map.get(&snort_url).unwrap(), &filter);
            }
            _ => panic!("Expected filters"),
        }

        // test orphan filters
        let random_keys = Keys::generate();
        let filter = Filter::new().author(random_keys.public_key);
        match graph
            .break_down_filter(filter.clone(), HashSet::new())
            .await
            .unwrap()
        {
            BrokenDownFilters::Orphan(f) => {
                assert_eq!(f, filter);
            }
            _ => panic!("Expected filters"),
        }
    }
}
