// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use nostr::prelude::*;
use tokio::sync::{RwLock, RwLockReadGuard};

use super::constant::PUBKEY_METADATA_OUTDATED_AFTER;

// TODO: add support to DM relay list

const P_TAG: SingleLetterTag = SingleLetterTag::lowercase(Alphabet::P);

#[derive(Debug)]
pub struct BrokenDownFilters {
    /// Filters by url
    pub filters: HashMap<Url, Vec<Filter>>,
    /// Filters that can be sent to read relays (generic query, not related to public keys)
    pub other: Vec<Filter>,
    pub outbox_urls: HashSet<Url>,
    pub inbox_urls: HashSet<Url>,
}

#[derive(Debug, Clone)]
struct RelayListMetadata {
    pub map: HashMap<Url, Option<RelayMetadata>>,
    /// Timestamp of when the event metadata was created
    pub event_created_at: Timestamp,
    /// Timestamp of when the metadata was updated
    pub last_update: Timestamp,
}

type PublicKeyMap = HashMap<PublicKey, RelayListMetadata>;

#[derive(Debug, Clone)]
pub struct GossipGraph {
    /// Keep track of seen public keys and of their NIP-65
    public_keys: Arc<RwLock<PublicKeyMap>>,
}

impl GossipGraph {
    pub fn new() -> Self {
        Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update graph
    pub async fn update<I>(&self, events: I)
    where
        I: IntoIterator<Item = Event>,
    {
        let mut public_keys = self.public_keys.write().await;

        for event in events.into_iter().filter(|e| e.kind == Kind::RelayList) {
            public_keys
                .entry(event.pubkey)
                .and_modify(|m| {
                    // Update only if new metadata has more recent timestamp
                    if event.created_at >= m.event_created_at {
                        *m = RelayListMetadata {
                            map: nip65::extract_relay_list(&event)
                                .map(|(u, m)| (u.clone(), *m))
                                .collect(),
                            event_created_at: event.created_at,
                            last_update: Timestamp::now(),
                        };
                    }
                })
                .or_insert_with(|| RelayListMetadata {
                    map: nip65::extract_relay_list(&event)
                        .map(|(u, m)| (u.clone(), *m))
                        .collect(),
                    event_created_at: event.created_at,
                    last_update: Timestamp::now(),
                });
        }
    }

    /// Check for what public keys the metadata are outdated or not existent
    pub async fn check_outdated<I>(&self, public_keys: I) -> HashSet<PublicKey>
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let map = self.public_keys.read().await;
        let now = Timestamp::now();

        let mut outdated: HashSet<PublicKey> = HashSet::new();

        for public_key in public_keys.into_iter() {
            match map.get(&public_key) {
                Some(meta) => {
                    let empty: bool = meta.map.is_empty();
                    let expired: bool = meta.last_update + PUBKEY_METADATA_OUTDATED_AFTER < now;

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

    fn get_nip65_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
        metadata: Option<RelayMetadata>,
    ) -> HashSet<Url>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut urls: HashSet<Url> = HashSet::new();

        for public_key in public_keys.into_iter() {
            if let Some(meta) = txn.get(public_key) {
                for (url, m) in meta.map.iter() {
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

    fn map_nip65_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
        metadata: RelayMetadata,
    ) -> HashMap<Url, BTreeSet<PublicKey>>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut urls: HashMap<Url, BTreeSet<PublicKey>> = HashMap::new();

        for public_key in public_keys.into_iter() {
            if let Some(meta) = txn.get(public_key) {
                for (url, m) in meta.map.iter() {
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
    pub async fn get_outbox_relays<'a, I>(&self, public_keys: I) -> HashSet<Url>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let txn = self.public_keys.read().await;
        self.get_nip65_relays(&txn, public_keys, Some(RelayMetadata::Write))
    }

    /// Get inbox (read) relays for public keys
    #[inline]
    pub async fn get_inbox_relays<'a, I>(&self, public_keys: I) -> HashSet<Url>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let txn = self.public_keys.read().await;
        self.get_nip65_relays(&txn, public_keys, Some(RelayMetadata::Read))
    }

    /// Map outbox (write) relays for public keys
    #[inline]
    fn map_outbox_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
    ) -> HashMap<Url, BTreeSet<PublicKey>>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        self.map_nip65_relays(txn, public_keys, RelayMetadata::Write)
    }

    /// Map inbox (read) relays for public keys
    #[inline]
    fn map_inbox_relays<'a, I>(
        &self,
        txn: &RwLockReadGuard<PublicKeyMap>,
        public_keys: I,
    ) -> HashMap<Url, BTreeSet<PublicKey>>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        self.map_nip65_relays(txn, public_keys, RelayMetadata::Read)
    }

    pub async fn break_down_filters(&self, filters: Vec<Filter>) -> BrokenDownFilters {
        let mut map: HashMap<Url, BTreeSet<Filter>> = HashMap::new();
        let mut other = Vec::new();
        let mut outbox_urls = HashSet::new();
        let mut inbox_urls = HashSet::new();

        let txn = self.public_keys.read().await;

        for filter in filters.into_iter() {
            // Extract `p` tag from generic tags and parse public key hex
            let p_tag: Option<BTreeSet<PublicKey>> = filter.generic_tags.get(&P_TAG).map(|s| {
                s.iter()
                    .filter_map(|p| PublicKey::from_hex(p).ok())
                    .collect()
            });

            match (&filter.authors, &p_tag) {
                (Some(authors), None) => {
                    // Get map of outbox relays
                    let outbox = self.map_outbox_relays(&txn, authors);

                    // Construct new filters
                    for (relay, pk_set) in outbox.into_iter() {
                        outbox_urls.insert(relay.clone());

                        // Clone filter and change authors
                        let mut new_filter: Filter = filter.clone();
                        new_filter.authors = Some(pk_set);

                        // Update map
                        map.entry(relay)
                            .and_modify(|f| {
                                f.insert(new_filter.clone());
                            })
                            .or_default()
                            .insert(new_filter);
                    }
                }
                (None, Some(p_public_keys)) => {
                    // Get map of inbox relays
                    let inbox = self.map_inbox_relays(&txn, p_public_keys);

                    // Construct new filters
                    for (relay, pk_set) in inbox.into_iter() {
                        inbox_urls.insert(relay.clone());

                        // Clone filter and change p tags
                        let mut new_filter: Filter = filter.clone();
                        new_filter
                            .generic_tags
                            .insert(P_TAG, pk_set.into_iter().map(|p| p.to_string()).collect());

                        // Update map
                        map.entry(relay)
                            .and_modify(|f| {
                                f.insert(new_filter.clone());
                            })
                            .or_default()
                            .insert(new_filter);
                    }
                }
                (Some(authors), Some(p_public_keys)) => {
                    // Get map of outbox and inbox relays
                    let pks = authors.union(p_public_keys);
                    let relays = self.get_nip65_relays(&txn, pks, None);

                    for relay in relays.into_iter() {
                        outbox_urls.insert(relay.clone());
                        inbox_urls.insert(relay.clone());

                        // Update map
                        map.entry(relay)
                            .and_modify(|f| {
                                f.insert(filter.clone());
                            })
                            .or_default()
                            .insert(filter.clone());
                    }
                }
                // Nothing to do, add to `other` list
                (None, None) => {
                    other.push(filter);
                }
            }
        }

        BrokenDownFilters {
            filters: map
                .into_iter()
                .map(|(u, f)| (u, f.into_iter().collect::<Vec<_>>()))
                .collect(),
            other,
            outbox_urls,
            inbox_urls,
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
            .filter_map(|(url, m)| Some((Url::parse(url).ok()?, m)));
        EventBuilder::relay_list(list).to_event(&keys).unwrap()
    }

    async fn setup_graph() -> GossipGraph {
        let graph = GossipGraph::new();

        let mut events = Vec::new();
        events.push(build_relay_list_event(SECRET_KEY_A, KEY_A_RELAYS.to_vec()));
        events.push(build_relay_list_event(SECRET_KEY_B, KEY_B_RELAYS.to_vec()));

        graph.update(events).await;

        graph
    }

    #[tokio::test]
    async fn test_scomposed_filters() {
        let keys_a = Keys::parse(SECRET_KEY_A).unwrap();
        let keys_b = Keys::parse(SECRET_KEY_B).unwrap();

        let damus_url = Url::parse("wss://relay.damus.io").unwrap();
        let nostr_bg_url = Url::parse("wss://relay.nostr.bg").unwrap();
        let nos_lol_url = Url::parse("wss://nos.lol").unwrap();
        let nostr_mom_url = Url::parse("wss://nostr.mom").unwrap();
        let nostr_info_url = Url::parse("wss://relay.nostr.info").unwrap();
        let relay_rip_url = Url::parse("wss://relay.rip").unwrap();
        let snort_url = Url::parse("wss://relay.snort.social").unwrap();

        let graph = setup_graph().await;

        // Single filter, single author
        let filters = vec![Filter::new().author(keys_a.public_key)];
        let scomposed = graph.break_down_filters(filters.clone()).await;

        assert_eq!(scomposed.filters.get(&damus_url).unwrap(), &filters);
        assert_eq!(scomposed.filters.get(&nostr_bg_url).unwrap(), &filters);
        assert_eq!(scomposed.filters.get(&nos_lol_url).unwrap(), &filters);
        assert!(!scomposed.filters.contains_key(&nostr_mom_url));
        assert!(scomposed.other.is_empty());

        // Multiple filters, multiple authors
        let authors_filter = Filter::new().authors([keys_a.public_key, keys_b.public_key]);
        let search_filter = Filter::new().search("Test").limit(10);
        let filters = vec![authors_filter.clone(), search_filter.clone()];
        let scomposed = graph.break_down_filters(filters.clone()).await;

        assert_eq!(
            scomposed.filters.get(&damus_url).unwrap(),
            &vec![authors_filter]
        );
        assert_eq!(
            scomposed.filters.get(&nostr_bg_url).unwrap(),
            &vec![Filter::new().author(keys_a.public_key)]
        );
        assert_eq!(
            scomposed.filters.get(&nos_lol_url).unwrap(),
            &vec![Filter::new().author(keys_a.public_key)]
        );
        assert!(!scomposed.filters.contains_key(&nostr_mom_url));
        assert_eq!(
            scomposed.filters.get(&nostr_info_url).unwrap(),
            &vec![Filter::new().author(keys_b.public_key)]
        );
        assert_eq!(
            scomposed.filters.get(&relay_rip_url).unwrap(),
            &vec![Filter::new().author(keys_b.public_key)]
        );
        assert!(!scomposed.filters.contains_key(&snort_url));
        assert_eq!(scomposed.other, vec![search_filter]);

        // Multiple filters, multiple authors and single p tags
        let authors_filter = Filter::new().authors([keys_a.public_key, keys_b.public_key]);
        let p_tag_filter = Filter::new().pubkey(keys_a.public_key);
        let search_filter = Filter::new().search("Test").limit(10);
        let filters = vec![
            authors_filter.clone(),
            p_tag_filter.clone(),
            search_filter.clone(),
        ];
        let scomposed = graph.break_down_filters(filters.clone()).await;

        assert_eq!(
            scomposed.filters.get(&damus_url).unwrap(),
            &vec![p_tag_filter.clone(), authors_filter]
        );
        assert_eq!(
            scomposed.filters.get(&nostr_bg_url).unwrap(),
            &vec![
                p_tag_filter.clone(),
                Filter::new().author(keys_a.public_key),
            ]
        );
        assert_eq!(
            scomposed.filters.get(&nos_lol_url).unwrap(),
            &vec![Filter::new().author(keys_a.public_key)]
        );
        assert_eq!(
            scomposed.filters.get(&nostr_mom_url).unwrap(),
            &vec![p_tag_filter]
        );
        assert_eq!(
            scomposed.filters.get(&nostr_info_url).unwrap(),
            &vec![Filter::new().author(keys_b.public_key)]
        );
        assert_eq!(
            scomposed.filters.get(&relay_rip_url).unwrap(),
            &vec![Filter::new().author(keys_b.public_key)]
        );
        assert!(!scomposed.filters.contains_key(&snort_url));
        assert_eq!(scomposed.other, vec![search_filter]);

        // Single filter, both author and p tag
        let filters = vec![Filter::new()
            .author(keys_a.public_key)
            .pubkey(keys_b.public_key)];
        let scomposed = graph.break_down_filters(filters.clone()).await;

        assert_eq!(scomposed.filters.get(&damus_url).unwrap(), &filters);
        assert_eq!(scomposed.filters.get(&nostr_bg_url).unwrap(), &filters);
        assert_eq!(scomposed.filters.get(&nos_lol_url).unwrap(), &filters);
        assert_eq!(scomposed.filters.get(&nostr_mom_url).unwrap(), &filters);
        assert_eq!(scomposed.filters.get(&nostr_info_url).unwrap(), &filters);
        assert_eq!(scomposed.filters.get(&relay_rip_url).unwrap(), &filters);
        assert_eq!(scomposed.filters.get(&snort_url).unwrap(), &filters);
        assert!(scomposed.other.is_empty());
    }
}
