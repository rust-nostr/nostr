// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use nostr::prelude::*;
use tokio::sync::{RwLock, RwLockReadGuard};

use super::constant::{CHECK_OUTDATED_INTERVAL, MAX_RELAYS_LIST, PUBKEY_METADATA_OUTDATED_AFTER};

const P_TAG: SingleLetterTag = SingleLetterTag::lowercase(Alphabet::P);

#[derive(Debug)]
pub enum BrokenDownFilters {
    /// Filters by url
    Filters(HashMap<RelayUrl, Filter>),
    /// Filters that match a certain pattern but where no relays are available
    Orphan(Filter),
    /// Filters that can be sent to read relays (generic query, not related to public keys)
    Other(Filter),
}

#[derive(Debug, Clone, Default)]
struct RelayList<T> {
    pub collection: T,
    /// Timestamp of when the event metadata was created
    pub event_created_at: Timestamp,
    /// Timestamp of when the metadata was updated
    pub last_update: Timestamp,
}

#[derive(Debug, Clone, Default)]
struct RelayLists {
    pub nip17: RelayList<HashSet<RelayUrl>>,
    pub nip65: RelayList<HashMap<RelayUrl, Option<RelayMetadata>>>,
    /// Timestamp of the last check
    pub last_check: Timestamp,
}

type PublicKeyMap = HashMap<PublicKey, RelayLists>;

#[derive(Debug, Clone)]
pub struct GossipGraph {
    /// Keep track of seen public keys and of their NIP65
    public_keys: Arc<RwLock<PublicKeyMap>>,
}

impl GossipGraph {
    pub fn new() -> Self {
        Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update graph
    ///
    /// Only the first [`MAX_RELAYS_LIST`] relays will be used.
    pub async fn update<I>(&self, events: I)
    where
        I: IntoIterator<Item = Event>,
    {
        let mut public_keys = self.public_keys.write().await;

        for event in events.into_iter() {
            if event.kind == Kind::RelayList {
                public_keys
                    .entry(event.pubkey)
                    .and_modify(|lists| {
                        // Update only if new metadata has more recent timestamp
                        if event.created_at >= lists.nip65.event_created_at {
                            lists.nip65 = RelayList {
                                collection: nip65::extract_relay_list(&event)
                                    .take(MAX_RELAYS_LIST)
                                    .map(|(u, m)| (u.clone(), *m))
                                    .collect(),
                                event_created_at: event.created_at,
                                last_update: Timestamp::now(),
                            };
                        }
                    })
                    .or_insert_with(|| RelayLists {
                        nip65: RelayList {
                            collection: nip65::extract_relay_list(&event)
                                .take(MAX_RELAYS_LIST)
                                .map(|(u, m)| (u.clone(), *m))
                                .collect(),
                            event_created_at: event.created_at,
                            last_update: Timestamp::now(),
                        },
                        ..Default::default()
                    });
            } else if event.kind == Kind::InboxRelays {
                public_keys
                    .entry(event.pubkey)
                    .and_modify(|lists| {
                        // Update only if new metadata has more recent timestamp
                        if event.created_at >= lists.nip17.event_created_at {
                            lists.nip17 = RelayList {
                                collection: nip17::extract_relay_list(&event)
                                    .take(MAX_RELAYS_LIST)
                                    .cloned()
                                    .collect(),
                                event_created_at: event.created_at,
                                last_update: Timestamp::now(),
                            };
                        }
                    })
                    .or_insert_with(|| RelayLists {
                        nip17: RelayList {
                            collection: nip17::extract_relay_list(&event)
                                .take(MAX_RELAYS_LIST)
                                .cloned()
                                .collect(),
                            event_created_at: event.created_at,
                            last_update: Timestamp::now(),
                        },
                        ..Default::default()
                    });
            }
        }
    }

    /// Check for what public keys the metadata are outdated or not existent (both for NIP17 and NIP65)
    pub async fn check_outdated<I>(&self, public_keys: I) -> HashSet<PublicKey>
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let map = self.public_keys.read().await;
        let now = Timestamp::now();

        let mut outdated: HashSet<PublicKey> = HashSet::new();

        for public_key in public_keys.into_iter() {
            match map.get(&public_key) {
                Some(lists) => {
                    if lists.last_check + CHECK_OUTDATED_INTERVAL > now {
                        continue;
                    }

                    // Check if collections are empty
                    let empty: bool =
                        lists.nip17.collection.is_empty() || lists.nip65.collection.is_empty();

                    // Check if expired
                    let expired: bool = lists.nip17.last_update + PUBKEY_METADATA_OUTDATED_AFTER
                        < now
                        || lists.nip65.last_update + PUBKEY_METADATA_OUTDATED_AFTER < now;

                    if empty || expired {
                        outdated.insert(public_key);
                    }
                }
                None => {
                    // Public key not found, insert into outdated
                    outdated.insert(public_key);
                }
            }
        }

        outdated
    }

    pub async fn update_last_check<I>(&self, public_keys: I)
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let mut map = self.public_keys.write().await;
        let now = Timestamp::now();

        for public_key in public_keys.into_iter() {
            map.entry(public_key)
                .and_modify(|lists| {
                    lists.last_check = now;
                })
                .or_insert_with(|| RelayLists {
                    last_check: now,
                    ..Default::default()
                });
        }
    }

    fn get_nip17_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
    ) -> HashSet<RelayUrl>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut urls: HashSet<RelayUrl> = HashSet::new();

        for public_key in public_keys.into_iter() {
            if let Some(lists) = txn.get(public_key) {
                for url in lists.nip17.collection.iter() {
                    urls.insert(url.clone());
                }
            }
        }

        urls
    }

    fn get_nip65_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
        metadata: Option<RelayMetadata>,
    ) -> HashSet<RelayUrl>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut urls: HashSet<RelayUrl> = HashSet::new();

        for public_key in public_keys.into_iter() {
            if let Some(lists) = txn.get(public_key) {
                for (url, m) in lists.nip65.collection.iter() {
                    let insert: bool = match m {
                        Some(val) => match metadata {
                            Some(metadata) => val == &metadata,
                            None => true,
                        },
                        None => true,
                    };

                    if insert {
                        urls.insert(url.clone());
                    }
                }
            }
        }

        urls
    }

    fn map_nip17_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
    ) -> HashMap<RelayUrl, BTreeSet<PublicKey>>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut urls: HashMap<RelayUrl, BTreeSet<PublicKey>> = HashMap::new();

        for public_key in public_keys.into_iter() {
            if let Some(lists) = txn.get(public_key) {
                for url in lists.nip17.collection.iter() {
                    urls.entry(url.clone())
                        .and_modify(|s| {
                            s.insert(*public_key);
                        })
                        .or_default()
                        .insert(*public_key);
                }
            }
        }

        urls
    }

    fn map_nip65_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
        metadata: RelayMetadata,
    ) -> HashMap<RelayUrl, BTreeSet<PublicKey>>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut urls: HashMap<RelayUrl, BTreeSet<PublicKey>> = HashMap::new();

        for public_key in public_keys.into_iter() {
            if let Some(lists) = txn.get(public_key) {
                for (url, m) in lists.nip65.collection.iter() {
                    let insert: bool = match m {
                        Some(val) => val == &metadata,
                        None => true,
                    };

                    if insert {
                        urls.entry(url.clone())
                            .and_modify(|s| {
                                s.insert(*public_key);
                            })
                            .or_default()
                            .insert(*public_key);
                    }
                }
            }
        }

        urls
    }

    /// Get outbox (write) relays for public keys
    #[inline]
    pub async fn get_nip65_outbox_relays<'a, I>(&self, public_keys: I) -> HashSet<RelayUrl>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let txn = self.public_keys.read().await;
        self.get_nip65_relays(&txn, public_keys, Some(RelayMetadata::Write))
    }

    /// Get inbox (read) relays for public keys
    #[inline]
    pub async fn get_nip65_inbox_relays<'a, I>(&self, public_keys: I) -> HashSet<RelayUrl>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let txn = self.public_keys.read().await;
        self.get_nip65_relays(&txn, public_keys, Some(RelayMetadata::Read))
    }

    /// Get NIP17 inbox (read) relays for public keys
    #[inline]
    pub async fn get_nip17_inbox_relays<'a, I>(&self, public_keys: I) -> HashSet<RelayUrl>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let txn = self.public_keys.read().await;
        self.get_nip17_relays(&txn, public_keys)
    }

    /// Map outbox (write) relays for public keys
    #[inline]
    fn map_nip65_outbox_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
    ) -> HashMap<RelayUrl, BTreeSet<PublicKey>>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        self.map_nip65_relays(txn, public_keys, RelayMetadata::Write)
    }

    /// Map NIP65 inbox (read) relays for public keys
    #[inline]
    fn map_nip65_inbox_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
    ) -> HashMap<RelayUrl, BTreeSet<PublicKey>>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        self.map_nip65_relays(txn, public_keys, RelayMetadata::Read)
    }

    pub async fn break_down_filter(&self, filter: Filter) -> BrokenDownFilters {
        let txn = self.public_keys.read().await;

        // Extract `p` tag from generic tags and parse public key hex
        let p_tag: Option<BTreeSet<PublicKey>> = filter.generic_tags.get(&P_TAG).map(|s| {
            s.iter()
                .filter_map(|p| PublicKey::from_hex(p).ok())
                .collect()
        });

        // Match pattern
        match (&filter.authors, &p_tag) {
            (Some(authors), None) => {
                // Get map of outbox relays
                let mut outbox: HashMap<RelayUrl, BTreeSet<PublicKey>> =
                    self.map_nip65_outbox_relays(&txn, authors);

                // Extend with NIP17 relays
                outbox.extend(self.map_nip17_relays(&txn, authors));

                // No relay available for the authors
                if outbox.is_empty() {
                    return BrokenDownFilters::Orphan(filter);
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

                BrokenDownFilters::Filters(map)
            }
            (None, Some(p_public_keys)) => {
                // Get map of inbox relays
                let mut inbox: HashMap<RelayUrl, BTreeSet<PublicKey>> =
                    self.map_nip65_inbox_relays(&txn, p_public_keys);

                // Extend with NIP17 relays
                inbox.extend(self.map_nip17_relays(&txn, p_public_keys));

                // No relay available for the p tags
                if inbox.is_empty() {
                    return BrokenDownFilters::Orphan(filter);
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

                BrokenDownFilters::Filters(map)
            }
            (Some(authors), Some(p_public_keys)) => {
                // Get map of outbox and inbox relays
                let mut relays: HashSet<RelayUrl> =
                    self.get_nip65_relays(&txn, authors.union(p_public_keys), None);

                // Extend with NIP17 relays
                relays.extend(self.get_nip17_relays(&txn, authors.union(p_public_keys)));

                // No relay available for the authors and p tags
                if relays.is_empty() {
                    return BrokenDownFilters::Orphan(filter);
                }

                let mut map: HashMap<RelayUrl, Filter> = HashMap::with_capacity(relays.len());

                for relay in relays.into_iter() {
                    // Update map
                    map.insert(relay, filter.clone());
                }

                BrokenDownFilters::Filters(map)
            }
            // Nothing to do, add to `other` list
            (None, None) => BrokenDownFilters::Other(filter),
        }
    }
}

#[cfg(test)]
mod tests {
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

    async fn setup_graph() -> GossipGraph {
        let graph = GossipGraph::new();

        let events = vec![
            build_relay_list_event(SECRET_KEY_A, KEY_A_RELAYS.to_vec()),
            build_relay_list_event(SECRET_KEY_B, KEY_B_RELAYS.to_vec()),
        ];

        graph.update(events).await;

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
        match graph.break_down_filter(filter.clone()).await {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &filter);
                assert_eq!(map.get(&nostr_bg_url).unwrap(), &filter);
                assert_eq!(map.get(&nos_lol_url).unwrap(), &filter);
                assert!(!map.contains_key(&nostr_mom_url));
            }
            _ => panic!("Expected filters"),
        }

        // Multiple authors
        let authors_filter = Filter::new().authors([keys_a.public_key, keys_b.public_key]);
        match graph.break_down_filter(authors_filter.clone()).await {
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
                assert!(!map.contains_key(&snort_url));
            }
            _ => panic!("Expected filters"),
        }

        // Other filter
        let search_filter = Filter::new().search("Test").limit(10);
        match graph.break_down_filter(search_filter.clone()).await {
            BrokenDownFilters::Other(filter) => {
                assert_eq!(filter, search_filter);
            }
            _ => panic!("Expected other"),
        }

        // Single p tags
        let p_tag_filter = Filter::new().pubkey(keys_a.public_key);
        match graph.break_down_filter(p_tag_filter.clone()).await {
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
        match graph.break_down_filter(filter.clone()).await {
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
        match graph.break_down_filter(filter.clone()).await {
            BrokenDownFilters::Orphan(f) => {
                assert_eq!(f, filter);
            }
            _ => panic!("Expected filters"),
        }
    }
}
