// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::hash_map::Entry as HashMapEntry;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use nostr::prelude::*;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod constant;

use self::constant::{
    CHECK_OUTDATED_INTERVAL, MAX_NIP17_RELAYS, MAX_RELAYS_ALLOWED_IN_NIP65,
    MAX_RELAYS_PER_NIP65_MARKER, PUBKEY_METADATA_OUTDATED_AFTER,
};

const P_TAG: SingleLetterTag = SingleLetterTag::lowercase(Alphabet::P);

pub(crate) enum GossipKind {
    Nip17,
    Nip65,
}

impl GossipKind {
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event_kind(self) -> Kind {
        match self {
            Self::Nip17 => Kind::InboxRelays,
            Self::Nip65 => Kind::RelayList,
        }
    }
}

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

/// Gossip tracker
#[derive(Debug, Clone)]
pub struct Gossip {
    /// Keep track of seen public keys and of their NIP65
    public_keys: Arc<RwLock<PublicKeyMap>>,
}

impl Gossip {
    pub fn new() -> Self {
        Self {
            public_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn process_event(&self, event: &Event) {
        // Check if the event can be processed
        // This avoids the acquire of the lock for every event processed that is not a NIP17 or NIP65
        if event.kind != Kind::RelayList && event.kind != Kind::InboxRelays {
            return;
        }

        // Acquire write lock
        let mut public_keys = self.public_keys.write().await;

        // Update
        self.update_event(&mut public_keys, event);
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
            self.update_event(&mut public_keys, &event);
        }
    }

    fn update_event(&self, public_keys: &mut RwLockWriteGuard<PublicKeyMap>, event: &Event) {
        if event.kind == Kind::RelayList {
            public_keys
                .entry(event.pubkey)
                .and_modify(|lists| {
                    // Update only if new metadata has more recent timestamp
                    if event.created_at >= lists.nip65.event_created_at {
                        lists.nip65 = RelayList {
                            collection: extract_nip65_relay_list(
                                event,
                                MAX_RELAYS_PER_NIP65_MARKER,
                            ),
                            event_created_at: event.created_at,
                            last_update: Timestamp::now(),
                        };
                    }
                })
                .or_insert_with(|| RelayLists {
                    nip65: RelayList {
                        collection: extract_nip65_relay_list(event, MAX_RELAYS_PER_NIP65_MARKER),
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
                            collection: nip17::extract_relay_list(event)
                                .take(MAX_NIP17_RELAYS)
                                .cloned()
                                .collect(),
                            event_created_at: event.created_at,
                            last_update: Timestamp::now(),
                        };
                    }
                })
                .or_insert_with(|| RelayLists {
                    nip17: RelayList {
                        collection: nip17::extract_relay_list(event)
                            .take(MAX_NIP17_RELAYS)
                            .cloned()
                            .collect(),
                        event_created_at: event.created_at,
                        last_update: Timestamp::now(),
                    },
                    ..Default::default()
                });
        }
    }

    /// Check for what public keys the metadata are outdated or not existent (both for NIP17 and NIP65)
    pub async fn check_outdated<I>(&self, public_keys: I, kind: &GossipKind) -> HashSet<PublicKey>
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

                    let (empty, expired) = match kind {
                        GossipKind::Nip17 => {
                            // Check if the collection is empty
                            let empty: bool = lists.nip17.collection.is_empty();

                            // Check if expired
                            let expired: bool =
                                lists.nip17.last_update + PUBKEY_METADATA_OUTDATED_AFTER < now;

                            (empty, expired)
                        }
                        GossipKind::Nip65 => {
                            // Check if the collection is empty
                            let empty: bool = lists.nip65.collection.is_empty();

                            // Check if expired
                            let expired: bool =
                                lists.nip65.last_update + PUBKEY_METADATA_OUTDATED_AFTER < now;

                            (empty, expired)
                        }
                    };

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

    pub async fn break_down_filter(
        &self,
        filter: Filter,
        pattern: GossipFilterPattern,
    ) -> BrokenDownFilters {
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
                if pattern.has_nip17() {
                    outbox.extend(self.map_nip17_relays(&txn, authors));
                }

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
                if pattern.has_nip17() {
                    inbox.extend(self.map_nip17_relays(&txn, p_public_keys));
                }

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
                if pattern.has_nip17() {
                    relays.extend(self.get_nip17_relays(&txn, authors.union(p_public_keys)));
                }

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

/// Extract at max `limit_per_marker` relays per NIP65 marker.
///
/// The output will be:
/// - `limit_per_marker` relays for `write`/`outbox`
/// - `limit_per_marker` relays for `read`/`inbox`
///
/// Some relays can be in common, reducing the number of the total max allowed relays.
///
/// Policy: give priority to relays that are used both for outbox and inbox.
fn extract_nip65_relay_list(
    event: &Event,
    limit_per_marker: usize,
) -> HashMap<RelayUrl, Option<RelayMetadata>> {
    // Use a vec to keep the relays in the same order of the event
    let mut both: Vec<RelayUrl> = Vec::new();
    let mut only_write: Vec<RelayUrl> = Vec::new();
    let mut only_read: Vec<RelayUrl> = Vec::new();

    for (url, meta) in nip65::extract_relay_list(event).take(MAX_RELAYS_ALLOWED_IN_NIP65) {
        match meta {
            Some(RelayMetadata::Write) => {
                only_write.push(url.clone());
            }
            Some(RelayMetadata::Read) => {
                only_read.push(url.clone());
            }
            None => {
                both.push(url.clone());
            }
        }
    }

    // Construct the map using the relays that cover both the read and write relays
    let mut map: HashMap<RelayUrl, Option<RelayMetadata>> = both
        .into_iter()
        .take(limit_per_marker)
        .map(|url| (url, None))
        .collect();

    let mut write_count: usize = map.len();
    let mut read_count: usize = map.len();

    // Check if there aren't enough write relays
    if write_count < limit_per_marker {
        for url in only_write.into_iter() {
            // Check if the limit is reached
            if write_count >= limit_per_marker {
                break;
            }

            // If the url doesn't exist, insert it
            if let HashMapEntry::Vacant(entry) = map.entry(url) {
                entry.insert(Some(RelayMetadata::Write));
                write_count += 1;
            }
        }
    }

    // Check if there aren't enough read relays
    if read_count < limit_per_marker {
        for url in only_read.into_iter() {
            // Check if the limit is reached
            if read_count >= limit_per_marker {
                break;
            }

            // Try to get relay
            match map.entry(url) {
                HashMapEntry::Occupied(mut entry) => {
                    // Check the metadata of the current entry
                    match entry.get() {
                        // The current entry already cover the write relay, upgrade it to cover both read and write.
                        Some(RelayMetadata::Write) => {
                            entry.insert(None);
                            read_count += 1;
                        }
                        // Duplicated entry, skip it
                        Some(RelayMetadata::Read) => continue,
                        // The current entry already cover the read relay, skip it
                        None => continue,
                    }
                }
                HashMapEntry::Vacant(entry) => {
                    entry.insert(Some(RelayMetadata::Read));
                    read_count += 1;
                }
            }
        }
    }

    map
}

pub(crate) enum GossipFilterPattern {
    Nip65,
    Nip65AndNip17,
}

impl GossipFilterPattern {
    #[inline]
    fn has_nip17(&self) -> bool {
        matches!(self, Self::Nip65AndNip17)
    }
}

/// Use both NIP-65 and NIP-17 if:
/// - the `kinds` field contains the [`Kind::GiftWrap`];
/// - if it's set a `#p` tag and no kind is specified
pub(crate) fn find_filter_pattern(filter: &Filter) -> GossipFilterPattern {
    let (are_kinds_empty, has_gift_wrap_kind): (bool, bool) = match &filter.kinds {
        Some(kinds) if kinds.is_empty() => (true, false),
        Some(kinds) => (false, kinds.contains(&Kind::GiftWrap)),
        None => (true, false),
    };
    let has_p_tags: bool = filter.generic_tags.contains_key(&P_TAG);

    // TODO: use both also if there are only IDs?

    if has_gift_wrap_kind || (has_p_tags && are_kinds_empty) {
        return GossipFilterPattern::Nip65AndNip17;
    }

    GossipFilterPattern::Nip65
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

    async fn setup_graph() -> Gossip {
        let graph = Gossip::new();

        let events = vec![
            build_relay_list_event(SECRET_KEY_A, KEY_A_RELAYS.to_vec()),
            build_relay_list_event(SECRET_KEY_B, KEY_B_RELAYS.to_vec()),
        ];

        graph.update(events).await;

        graph
    }

    fn count_extracted_nip65_relays(
        result: &HashMap<RelayUrl, Option<RelayMetadata>>,
    ) -> (usize, usize) {
        // Count final markers
        let mut write_count = 0;
        let mut read_count = 0;

        for metadata in result.values() {
            match metadata {
                Some(RelayMetadata::Write) => write_count += 1,
                Some(RelayMetadata::Read) => read_count += 1,
                None => {
                    write_count += 1;
                    read_count += 1;
                }
            }
        }

        (write_count, read_count)
    }

    #[test]
    fn test_extract_nip65_relay_list_priority() {
        let relays = vec![
            ("wss://relay1.com", None), // Both read and write
            ("wss://relay2.com", Some(RelayMetadata::Write)),
            ("wss://relay3.com", Some(RelayMetadata::Read)),
            ("wss://relay4.com", Some(RelayMetadata::Write)),
            ("wss://relay5.com", None), // Both read and write
            ("wss://relay6.com", Some(RelayMetadata::Read)),
            ("wss://relay7.com", Some(RelayMetadata::Read)),
            ("wss://relay8.com", None), // Both read and write
        ];

        let event = build_relay_list_event(SECRET_KEY_A, relays);
        let result = extract_nip65_relay_list(&event, 3);

        // Count final markers
        let (write_count, read_count) = count_extracted_nip65_relays(&result);

        assert_eq!(write_count, 3);
        assert_eq!(read_count, 3);

        // Extract only the relays with metadata set to None, following the priority policy
        let relay1_url = RelayUrl::parse("wss://relay1.com").unwrap();
        assert!(result.contains_key(&relay1_url));
        let relay5_url = RelayUrl::parse("wss://relay5.com").unwrap();
        assert!(result.contains_key(&relay5_url));
        let relay8_url = RelayUrl::parse("wss://relay8.com").unwrap();
        assert!(result.contains_key(&relay8_url));
    }

    #[test]
    fn test_extract_nip65_relay_list_priority_2() {
        let relays = vec![
            ("wss://relay1.com", None), // Both read and write
            ("wss://relay2.com", Some(RelayMetadata::Write)),
            ("wss://relay3.com", Some(RelayMetadata::Read)),
            ("wss://relay4.com", Some(RelayMetadata::Write)),
            ("wss://relay6.com", Some(RelayMetadata::Read)),
            ("wss://relay7.com", Some(RelayMetadata::Read)), // 4th read relay, must not be included
            ("wss://relay8.com", Some(RelayMetadata::Write)), // 4th write relay, must not be included
        ];

        let event = build_relay_list_event(SECRET_KEY_A, relays);
        let result = extract_nip65_relay_list(&event, 3);

        // Count final markers
        let (write_count, read_count) = count_extracted_nip65_relays(&result);

        assert_eq!(write_count, 3);
        assert_eq!(read_count, 3);
        assert_eq!(result.len(), 5); // 1 that cover both + 2 write only + 2 read only

        let relay1_url = RelayUrl::parse("wss://relay1.com").unwrap();
        assert_eq!(result.get(&relay1_url), Some(&None));

        let relay2_url = RelayUrl::parse("wss://relay2.com").unwrap();
        assert_eq!(result.get(&relay2_url), Some(&Some(RelayMetadata::Write)));

        let relay3_url = RelayUrl::parse("wss://relay3.com").unwrap();
        assert_eq!(result.get(&relay3_url), Some(&Some(RelayMetadata::Read)));

        let relay4_url = RelayUrl::parse("wss://relay4.com").unwrap();
        assert_eq!(result.get(&relay4_url), Some(&Some(RelayMetadata::Write)));

        let relay6_url = RelayUrl::parse("wss://relay6.com").unwrap();
        assert_eq!(result.get(&relay6_url), Some(&Some(RelayMetadata::Read)));
    }

    #[test]
    fn test_extract_nip65_relay_list_merging() {
        let relays = vec![
            ("wss://relay1.com", Some(RelayMetadata::Write)),
            ("wss://relay1.com", Some(RelayMetadata::Read)),
        ];

        let event = build_relay_list_event(SECRET_KEY_A, relays);
        let result = extract_nip65_relay_list(&event, 3);

        // Count final markers
        let (write_count, read_count) = count_extracted_nip65_relays(&result);

        assert_eq!(write_count, 1);
        assert_eq!(read_count, 1);
        assert_eq!(result.len(), 1); // 1 that cover both

        let relay1_url = RelayUrl::parse("wss://relay1.com").unwrap();
        assert_eq!(result.get(&relay1_url), Some(&None));
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
            .break_down_filter(filter.clone(), GossipFilterPattern::Nip65)
            .await
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
        let authors_filter = Filter::new().authors([keys_a.public_key, keys_b.public_key]);
        match graph
            .break_down_filter(authors_filter.clone(), GossipFilterPattern::Nip65)
            .await
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
                assert!(!map.contains_key(&snort_url));
            }
            _ => panic!("Expected filters"),
        }

        // Other filter
        let search_filter = Filter::new().search("Test").limit(10);
        match graph
            .break_down_filter(search_filter.clone(), GossipFilterPattern::Nip65)
            .await
        {
            BrokenDownFilters::Other(filter) => {
                assert_eq!(filter, search_filter);
            }
            _ => panic!("Expected other"),
        }

        // Single p tags
        let p_tag_filter = Filter::new().pubkey(keys_a.public_key);
        match graph
            .break_down_filter(p_tag_filter.clone(), GossipFilterPattern::Nip65)
            .await
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
            .break_down_filter(filter.clone(), GossipFilterPattern::Nip65)
            .await
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
            .break_down_filter(filter.clone(), GossipFilterPattern::Nip65)
            .await
        {
            BrokenDownFilters::Orphan(f) => {
                assert_eq!(f, filter);
            }
            _ => panic!("Expected filters"),
        }
    }
}
