// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;

use nostr::prelude::*;
use nostr_gossip::{BestRelaySelection, NostrGossip};

use crate::client::Error;

const P_TAG: SingleLetterTag = SingleLetterTag::lowercase(Alphabet::P);
const MAX_NIP17_RELAYS: usize = 3;

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
pub(crate) struct GossipWrapper {
    gossip: Arc<dyn NostrGossip>,
}

impl Deref for GossipWrapper {
    type Target = Arc<dyn NostrGossip>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.gossip
    }
}

impl GossipWrapper {
    #[inline]
    pub(crate) fn new(gossip: Arc<dyn NostrGossip>) -> Self {
        Self { gossip }
    }

    pub(crate) async fn get_relays<'a, I>(
        &self,
        public_keys: I,
        selection: BestRelaySelection,
    ) -> Result<HashSet<RelayUrl>, Error>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut urls: HashSet<RelayUrl> = HashSet::new();

        for public_key in public_keys.into_iter() {
            let relays: HashSet<RelayUrl> =
                self.gossip.get_best_relays(public_key, selection).await?;
            urls.extend(relays);
        }

        Ok(urls)
    }

    async fn map_relays<'a, I>(
        &self,
        public_keys: I,
        selection: BestRelaySelection,
    ) -> Result<HashMap<RelayUrl, BTreeSet<PublicKey>>, Error>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let mut urls: HashMap<RelayUrl, BTreeSet<PublicKey>> = HashMap::new();

        for public_key in public_keys.into_iter() {
            let relays: HashSet<RelayUrl> =
                self.gossip.get_best_relays(public_key, selection).await?;

            for url in relays.into_iter() {
                urls.entry(url)
                    .and_modify(|s| {
                        s.insert(*public_key);
                    })
                    .or_default()
                    .insert(*public_key);
            }
        }

        Ok(urls)
    }

    pub(crate) async fn break_down_filter(
        &self,
        filter: Filter,
        pattern: GossipFilterPattern,
    ) -> Result<BrokenDownFilters, Error> {
        // Extract `p` tag from generic tags and parse public key hex
        let p_tag: Option<BTreeSet<PublicKey>> = filter.generic_tags.get(&P_TAG).map(|s| {
            s.iter()
                .filter_map(|p| PublicKey::from_hex(p).ok())
                .collect()
        });

        // Match pattern
        match (&filter.authors, &p_tag) {
            (Some(authors), None) => {
                // Get map of write relays
                let mut outbox: HashMap<RelayUrl, BTreeSet<PublicKey>> = self
                    .map_relays(authors, BestRelaySelection::Write { limit: 2 })
                    .await?;

                // Get map of hints relays
                let hints: HashMap<RelayUrl, BTreeSet<PublicKey>> = self
                    .map_relays(authors, BestRelaySelection::Hints { limit: 1 })
                    .await?;

                // Get map of relays that received more events
                let most_received: HashMap<RelayUrl, BTreeSet<PublicKey>> = self
                    .map_relays(authors, BestRelaySelection::MostReceived { limit: 1 })
                    .await?;

                // Extend with hints and most received
                outbox.extend(hints);
                outbox.extend(most_received);

                if pattern.has_nip17() {
                    // Get map of private message relays
                    let nip17_relays = self
                        .map_relays(
                            authors,
                            BestRelaySelection::PrivateMessage {
                                limit: MAX_NIP17_RELAYS,
                            },
                        )
                        .await?;

                    outbox.extend(nip17_relays);
                }

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
                // Get map of inbox relays
                let mut inbox: HashMap<RelayUrl, BTreeSet<PublicKey>> = self
                    .map_relays(p_public_keys, BestRelaySelection::Read { limit: 2 })
                    .await?;

                // Get map of hints relays
                let hints: HashMap<RelayUrl, BTreeSet<PublicKey>> = self
                    .map_relays(p_public_keys, BestRelaySelection::Hints { limit: 1 })
                    .await?;

                // Get map of relays that received more events
                let most_received: HashMap<RelayUrl, BTreeSet<PublicKey>> = self
                    .map_relays(p_public_keys, BestRelaySelection::MostReceived { limit: 1 })
                    .await?;

                // Extend with hints and most received
                inbox.extend(hints);
                inbox.extend(most_received);

                // Extend with NIP17 relays
                if pattern.has_nip17() {
                    // Get map of private message relays
                    let nip17_relays = self
                        .map_relays(
                            p_public_keys,
                            BestRelaySelection::PrivateMessage {
                                limit: MAX_NIP17_RELAYS,
                            },
                        )
                        .await?;

                    inbox.extend(nip17_relays);
                }

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
                let union: BTreeSet<PublicKey> = authors.union(p_public_keys).copied().collect();

                // Get map of outbox and inbox relays
                let mut relays: HashSet<RelayUrl> = self
                    .get_relays(
                        union.iter(),
                        BestRelaySelection::All {
                            read: 2,
                            write: 2,
                            hints: 1,
                            most_received: 1,
                        },
                    )
                    .await?;

                // Extend with NIP17 relays
                if pattern.has_nip17() {
                    // Get map of private message relays
                    let nip17_relays = self
                        .get_relays(
                            union.iter(),
                            BestRelaySelection::PrivateMessage {
                                limit: MAX_NIP17_RELAYS,
                            },
                        )
                        .await?;

                    relays.extend(nip17_relays);
                }

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
    use nostr_gossip_memory::prelude::*;

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

    async fn setup() -> GossipWrapper {
        let db = NostrGossipMemory::unbounded();

        let events = vec![
            build_relay_list_event(SECRET_KEY_A, KEY_A_RELAYS.to_vec()),
            build_relay_list_event(SECRET_KEY_B, KEY_B_RELAYS.to_vec()),
        ];

        for event in events {
            db.process(&event, None).await.unwrap();
        }

        GossipWrapper::new(Arc::new(db))
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

        let gossip = setup().await;

        // Single author
        let filter = Filter::new().author(keys_a.public_key);
        match gossip
            .break_down_filter(filter.clone(), GossipFilterPattern::Nip65)
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &filter);
                assert_eq!(map.get(&nostr_bg_url).unwrap(), &filter);
                //assert_eq!(map.get(&nos_lol_url).unwrap(), &filter); // Not contains this because the limit is 2 relays + hints and most received
                assert!(!map.contains_key(&nostr_mom_url));
            }
            _ => panic!("Expected filters"),
        }

        // Multiple authors
        let authors_filter = Filter::new().authors([keys_a.public_key, keys_b.public_key]);
        match gossip
            .break_down_filter(authors_filter.clone(), GossipFilterPattern::Nip65)
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &authors_filter);
                assert_eq!(
                    map.get(&nostr_bg_url).unwrap(),
                    &Filter::new().author(keys_a.public_key)
                );
                // assert_eq!(
                //     map.get(&nos_lol_url).unwrap(),
                //     &Filter::new().author(keys_a.public_key)
                // );
                assert!(!map.contains_key(&nostr_mom_url));
                assert_eq!(
                    map.get(&nostr_info_url).unwrap(),
                    &Filter::new().author(keys_b.public_key)
                );
                // assert_eq!(
                //     map.get(&relay_rip_url).unwrap(),
                //     &Filter::new().author(keys_b.public_key)
                // );
                assert!(!map.contains_key(&snort_url));
            }
            _ => panic!("Expected filters"),
        }

        // Other filter
        let search_filter = Filter::new().search("Test").limit(10);
        match gossip
            .break_down_filter(search_filter.clone(), GossipFilterPattern::Nip65)
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
        match gossip
            .break_down_filter(p_tag_filter.clone(), GossipFilterPattern::Nip65)
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &p_tag_filter);
                assert_eq!(map.get(&nostr_bg_url).unwrap(), &p_tag_filter);
                //assert_eq!(map.get(&nostr_mom_url).unwrap(), &p_tag_filter);
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
        match gossip
            .break_down_filter(filter.clone(), GossipFilterPattern::Nip65)
            .await
            .unwrap()
        {
            BrokenDownFilters::Filters(map) => {
                assert_eq!(map.get(&damus_url).unwrap(), &filter);
                assert_eq!(map.get(&nostr_bg_url).unwrap(), &filter);
                //assert_eq!(map.get(&nos_lol_url).unwrap(), &filter);
                //assert_eq!(map.get(&nostr_mom_url).unwrap(), &filter);
                //assert_eq!(map.get(&nostr_info_url).unwrap(), &filter);
                //assert_eq!(map.get(&relay_rip_url).unwrap(), &filter);
                //assert_eq!(map.get(&snort_url).unwrap(), &filter);
            }
            _ => panic!("Expected filters"),
        }

        // test orphan filters
        let random_keys = Keys::generate();
        let filter = Filter::new().author(random_keys.public_key);
        match gossip
            .break_down_filter(filter.clone(), GossipFilterPattern::Nip65)
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
