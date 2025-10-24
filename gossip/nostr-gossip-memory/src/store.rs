//! Gossip in-memory storage.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use nostr::nips::nip17;
use nostr::nips::nip65::{self, RelayMetadata};
use nostr::util::BoxedFuture;
use nostr::{Event, Kind, PublicKey, RelayUrl, TagKind, TagStandard, Timestamp};
use nostr_gossip::error::GossipError;
use nostr_gossip::{BestRelaySelection, GossipListKind, GossipPublicKeyStatus, NostrGossip};
use tokio::sync::Mutex;

use crate::constant::PUBKEY_METADATA_OUTDATED_AFTER;
use crate::flags::Flags;

#[derive(Default)]
struct PkRelayData {
    bitflags: Flags,
    received_events: u64,
    last_received_event: Option<Timestamp>,
}

struct PkData {
    last_nip17_update: Option<Timestamp>,
    last_nip65_update: Option<Timestamp>,
    relays: LruCache<RelayUrl, PkRelayData>,
}

impl Default for PkData {
    fn default() -> Self {
        Self {
            last_nip17_update: None,
            last_nip65_update: None,
            relays: LruCache::new(NonZeroUsize::new(25).expect("Invalid cache size")),
        }
    }
}

/// Gossip in-memory storage.
#[derive(Debug, Clone)]
pub struct NostrGossipMemory {
    public_keys: Arc<Mutex<LruCache<PublicKey, PkData>>>,
}

impl Default for NostrGossipMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl NostrGossipMemory {
    /// Construct a new instance
    pub fn new() -> Self {
        Self {
            // TODO: allow to make this bounded
            public_keys: Arc::new(Mutex::new(LruCache::unbounded())),
        }
    }

    async fn process_event(&self, event: &Event, relay_url: Option<&RelayUrl>) {
        let mut public_keys = self.public_keys.lock().await;

        match &event.kind {
            // Extract NIP-65 relays
            Kind::RelayList => {
                let pk_data: &mut PkData =
                    public_keys.get_or_insert_mut(event.pubkey, PkData::default);

                for (relay_url, metadata) in nip65::extract_relay_list(event) {
                    // New bitflag for the relay
                    let bitflag: Flags = match metadata {
                        Some(RelayMetadata::Read) => Flags::READ,
                        Some(RelayMetadata::Write) => Flags::WRITE,
                        None => {
                            let mut f = Flags::READ;
                            f.add(Flags::WRITE);
                            f
                        }
                    };

                    // Create a mask for READ and WRITE flags
                    let mut read_write_mask: Flags = Flags::READ;
                    read_write_mask.add(Flags::WRITE);

                    let relay_data = pk_data
                        .relays
                        .get_or_insert_mut(relay_url.clone(), PkRelayData::default);

                    // Update the bitflag: remove the previous READ and WRITE values and apply the new bitflag (preserves any other flag)
                    relay_data.bitflags.remove(read_write_mask);
                    relay_data.bitflags.add(bitflag);
                }
            }
            // Extract NIP-17 relays
            Kind::InboxRelays => {
                let pk_data: &mut PkData =
                    public_keys.get_or_insert_mut(event.pubkey, PkData::default);

                for relay_url in nip17::extract_relay_list(event) {
                    let relay_data = pk_data
                        .relays
                        .get_or_insert_mut(relay_url.clone(), PkRelayData::default);

                    relay_data.bitflags.add(Flags::PRIVATE_MESSAGE);
                }
            }
            // Extract hints
            _ => {
                for tag in event.tags.filter_standardized(TagKind::p()) {
                    if let TagStandard::PublicKey {
                        public_key,
                        relay_url: Some(relay_url),
                        ..
                    } = tag
                    {
                        let pk_data: &mut PkData =
                            public_keys.get_or_insert_mut(*public_key, PkData::default);
                        update_relay_per_user(pk_data, relay_url.clone(), Flags::HINT);
                    }
                }
            }
        }

        if let Some(relay_url) = relay_url {
            let pk_data: &mut PkData = public_keys.get_or_insert_mut(event.pubkey, PkData::default);
            update_relay_per_user(pk_data, relay_url.clone(), Flags::RECEIVED);
        }
    }

    async fn get_status(
        &self,
        public_key: &PublicKey,
        list: GossipListKind,
    ) -> GossipPublicKeyStatus {
        let mut public_keys = self.public_keys.lock().await;

        match public_keys.get(public_key) {
            Some(pk_data) => {
                let now: Timestamp = Timestamp::now();

                match (list, pk_data.last_nip17_update, pk_data.last_nip65_update) {
                    (GossipListKind::Nip17, Some(last), _) => {
                        if last + PUBKEY_METADATA_OUTDATED_AFTER < now {
                            GossipPublicKeyStatus::Outdated { created_at: None }
                        } else {
                            GossipPublicKeyStatus::Updated
                        }
                    }
                    (GossipListKind::Nip65, _, Some(last)) => {
                        if last + PUBKEY_METADATA_OUTDATED_AFTER < now {
                            GossipPublicKeyStatus::Outdated { created_at: None }
                        } else {
                            GossipPublicKeyStatus::Updated
                        }
                    }
                    (_, _, _) => GossipPublicKeyStatus::Outdated { created_at: None },
                }
            }
            None => GossipPublicKeyStatus::Outdated { created_at: None },
        }
    }

    async fn _update_fetch_attempt(&self, public_key: &PublicKey, list: GossipListKind) {
        let mut public_keys = self.public_keys.lock().await;

        let pk_data: &mut PkData = public_keys.get_or_insert_mut(*public_key, PkData::default);

        let now: Timestamp = Timestamp::now();

        match list {
            GossipListKind::Nip17 => pk_data.last_nip17_update = Some(now),
            GossipListKind::Nip65 => pk_data.last_nip65_update = Some(now),
        };
    }

    async fn _get_best_relays(
        &self,
        public_key: &PublicKey,
        selection: BestRelaySelection,
    ) -> HashSet<RelayUrl> {
        let public_keys = self.public_keys.lock().await;

        let mut relays: HashSet<RelayUrl> = HashSet::new();

        match selection {
            BestRelaySelection::All {
                read,
                write,
                hints,
                most_received,
            } => {
                // Get read relays
                relays.extend(self.get_relays_by_flag(&public_keys, public_key, Flags::READ, read));

                // Get write relays
                relays.extend(self.get_relays_by_flag(
                    &public_keys,
                    public_key,
                    Flags::WRITE,
                    write,
                ));

                // Get hint relays
                relays.extend(self.get_relays_by_flag(
                    &public_keys,
                    public_key,
                    Flags::HINT,
                    hints,
                ));

                // Get most received relays
                relays.extend(self.get_relays_by_flag(
                    &public_keys,
                    public_key,
                    Flags::RECEIVED,
                    most_received,
                ));
            }
            BestRelaySelection::Read { limit } => {
                relays.extend(self.get_relays_by_flag(
                    &public_keys,
                    public_key,
                    Flags::READ,
                    limit,
                ));
            }
            BestRelaySelection::Write { limit } => {
                relays.extend(self.get_relays_by_flag(
                    &public_keys,
                    public_key,
                    Flags::WRITE,
                    limit,
                ));
            }
            BestRelaySelection::PrivateMessage { limit } => {
                relays.extend(self.get_relays_by_flag(
                    &public_keys,
                    public_key,
                    Flags::PRIVATE_MESSAGE,
                    limit,
                ));
            }
            BestRelaySelection::Hints { limit } => {
                relays.extend(self.get_relays_by_flag(
                    &public_keys,
                    public_key,
                    Flags::HINT,
                    limit,
                ));
            }
            BestRelaySelection::MostReceived { limit } => {
                relays.extend(self.get_relays_by_flag(
                    &public_keys,
                    public_key,
                    Flags::RECEIVED,
                    limit,
                ));
            }
        }

        relays
    }

    fn get_relays_by_flag(
        &self,
        tx: &LruCache<PublicKey, PkData>,
        public_key: &PublicKey,
        flag: Flags,
        limit: usize,
    ) -> impl Iterator<Item = RelayUrl> + '_ {
        let mut relays: Vec<(RelayUrl, u64, Option<Timestamp>)> = Vec::new();

        if let Some(pk_data) = tx.peek(public_key) {
            for (relay_url, relay_data) in pk_data.relays.iter() {
                // Check if the relay has the specified flag
                if relay_data.bitflags.has(flag) {
                    relays.push((
                        relay_url.clone(),
                        relay_data.received_events,
                        relay_data.last_received_event,
                    ));
                }
            }
        }

        // Sort by received_events DESC, then by last_received_event DESC
        relays.sort_by(|a, b| match b.1.cmp(&a.1) {
            Ordering::Equal => b.2.cmp(&a.2),
            other => other,
        });

        // Take only the requested limit and extract relay URLs
        relays.into_iter().take(limit).map(|(url, _, _)| url)
    }
}

/// Add relay per user or update the received events and bitflags.
fn update_relay_per_user(pk_data: &mut PkData, relay_url: RelayUrl, flags: Flags) {
    let relay_data = pk_data
        .relays
        .get_or_insert_mut(relay_url, PkRelayData::default);

    relay_data.bitflags.add(flags);
    relay_data.received_events = relay_data.received_events.saturating_add(1);
    relay_data.last_received_event = Some(Timestamp::now());
}

impl NostrGossip for NostrGossipMemory {
    fn process<'a>(
        &'a self,
        event: &'a Event,
        relay_url: Option<&'a RelayUrl>,
    ) -> BoxedFuture<'a, Result<(), GossipError>> {
        Box::pin(async move {
            self.process_event(event, relay_url).await;
            Ok(())
        })
    }

    fn status<'a>(
        &'a self,
        public_key: &'a PublicKey,
        list: GossipListKind,
    ) -> BoxedFuture<'a, Result<GossipPublicKeyStatus, GossipError>> {
        Box::pin(async move { Ok(self.get_status(public_key, list).await) })
    }

    fn update_fetch_attempt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        list: GossipListKind,
    ) -> BoxedFuture<'a, Result<(), GossipError>> {
        Box::pin(async move {
            self._update_fetch_attempt(public_key, list).await;
            Ok(())
        })
    }

    fn get_best_relays<'a>(
        &'a self,
        public_key: &'a PublicKey,
        selection: BestRelaySelection,
    ) -> BoxedFuture<'a, Result<HashSet<RelayUrl>, GossipError>> {
        Box::pin(async move { Ok(self._get_best_relays(public_key, selection).await) })
    }
}

#[cfg(test)]
mod tests {
    use nostr::{EventBuilder, JsonUtil, Keys, Tag};

    use super::*;

    #[tokio::test]
    async fn test_process_event() {
        let store = NostrGossipMemory::default();

        let json = r#"{"id":"b7b1fb52ad8461a03e949820ae29a9ea07e35bcd79c95c4b59b0254944f62805","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644581,"kind":1,"tags":[],"content":"Text note","sig":"ed73a8a4e7c26cd797a7b875c634d9ecb6958c57733305fed23b978109d0411d21b3e182cb67c8ad750884e30ca383b509382ae6187b36e76ee76e6a142c4284"}"#;
        let event = Event::from_json(json).unwrap();

        // First process
        store.process(&event, None).await.unwrap();

        // Re-process the same event
        store.process(&event, None).await.unwrap();
    }

    #[tokio::test]
    async fn test_process_nip65_relay_list() {
        let store = NostrGossipMemory::default();

        // NIP-65 relay list event with read and write relays
        let json = r#"{"id":"0a49bed4a1eb0973a68a0d43b7ca62781ffd4e052b91bbadef09e5cf756f6e68","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1759351841,"kind":10002,"tags":[["alt","Relay list to discover the user's content"],["r","wss://relay.damus.io/"],["r","wss://nostr.wine/"],["r","wss://nostr.oxtr.dev/"],["r","wss://relay.nostr.wirednet.jp/"]],"content":"","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let event = Event::from_json(json).unwrap();

        store.process(&event, None).await.unwrap();

        let public_key = event.pubkey;

        // Test Read selection
        let read_relays = store
            ._get_best_relays(&public_key, BestRelaySelection::Read { limit: 2 })
            .await;

        assert_eq!(read_relays.len(), 2); // relay.damus.io and nos.lol

        // Test Write selection
        let write_relays = store
            ._get_best_relays(&public_key, BestRelaySelection::Write { limit: 2 })
            .await;

        assert_eq!(write_relays.len(), 2); // relay.damus.io and relay.nostr.band
    }

    #[tokio::test]
    async fn test_process_nip17_inbox_relays() {
        let store = NostrGossipMemory::default();

        // NIP-17 inbox relays event
        let json = r#"{"id":"8d9b40907f80bd7d5014bdc6a2541227b92f4ae20cbff59792b4746a713da81e","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1756718818,"kind":10050,"tags":[["relay","wss://auth.nostr1.com/"],["relay","wss://nostr.oxtr.dev/"],["relay","wss://nip17.com"]],"content":"","sig":"05611df32f5c4e55bb8d74ab2840378b7707ad162f785a78f8bdaecee5b872667e4e43bcbbf3c6c638335c637f001155b48b7a7040ce2695660467be62f142d5"}"#;
        let event = Event::from_json(json).unwrap();

        store.process(&event, None).await.unwrap();

        let public_key = event.pubkey;

        // Test PrivateMessage selection
        let pm_relays = store
            ._get_best_relays(&public_key, BestRelaySelection::PrivateMessage { limit: 4 })
            .await;

        assert_eq!(pm_relays.len(), 3); // inbox.nostr.wine and relay.primal.net
    }

    #[tokio::test]
    async fn test_process_hints_from_p_tags() {
        let store = NostrGossipMemory::default();

        let public_key =
            PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")
                .unwrap();
        let relay_url = RelayUrl::parse("wss://hint.relay.io").unwrap();

        let keys = Keys::generate();
        let event = EventBuilder::text_note("test")
            .tag(Tag::from_standardized_without_cell(
                TagStandard::PublicKey {
                    public_key,
                    relay_url: Some(relay_url.clone()),
                    alias: None,
                    uppercase: false,
                },
            ))
            .sign_with_keys(&keys)
            .unwrap();

        store.process(&event, None).await.unwrap();

        let hint_relays = store
            ._get_best_relays(&public_key, BestRelaySelection::Hints { limit: 5 })
            .await;

        assert_eq!(hint_relays.len(), 1);
        assert!(hint_relays.iter().any(|r| r == &relay_url));
    }

    #[tokio::test]
    async fn test_received_events_tracking() {
        let store = NostrGossipMemory::default();

        let keys = Keys::generate();
        let relay_url = RelayUrl::parse("wss://test.relay.io").unwrap();

        // Process multiple events from the same relay
        for i in 0..5 {
            let event = EventBuilder::text_note(format!("Test {i}"))
                .sign_with_keys(&keys)
                .unwrap();

            store.process(&event, Some(&relay_url)).await.unwrap();
        }

        // Test MostReceived selection
        let most_received = store
            ._get_best_relays(
                &keys.public_key,
                BestRelaySelection::MostReceived { limit: 10 },
            )
            .await;

        assert_eq!(most_received.len(), 1);
        assert!(most_received.iter().any(|r| r == &relay_url));
    }

    #[tokio::test]
    async fn test_best_relays_all_selection() {
        let store = NostrGossipMemory::default();

        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();

        // Add NIP-65 relays
        let nip65_json = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000000","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1704644581,"kind":10002,"tags":[["r","wss://read.relay.io","read"],["r","wss://write.relay.io","write"]],"content":"","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let nip65_event = Event::from_json(nip65_json).unwrap();
        store.process(&nip65_event, None).await.unwrap();

        // Add event with hints
        let hint_json = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000001","pubkey":"bb4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644581,"kind":1,"tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","wss://hint.relay.io"]],"content":"Hint","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let hint_event = Event::from_json(hint_json).unwrap();
        store.process(&hint_event, None).await.unwrap();

        // Add received events
        let relay_url = RelayUrl::parse("wss://received.relay.io").unwrap();
        let received_json = r#"{"id":"0000000000000000000000000000000000000000000000000000000000000002","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1704644581,"kind":1,"tags":[],"content":"Received","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let received_event = Event::from_json(received_json).unwrap();
        store
            .process(&received_event, Some(&relay_url))
            .await
            .unwrap();

        // Test All selection
        let all_relays = store
            ._get_best_relays(
                &public_key,
                BestRelaySelection::All {
                    read: 5,
                    write: 5,
                    hints: 5,
                    most_received: 5,
                },
            )
            .await;

        // Should have relays from all categories (duplicates removed by HashSet)
        assert!(all_relays.len() >= 3);
        assert!(all_relays
            .iter()
            .any(|r| r.as_str() == "wss://read.relay.io"));
        assert!(all_relays
            .iter()
            .any(|r| r.as_str() == "wss://write.relay.io"));
        assert!(all_relays
            .iter()
            .any(|r| r.as_str() == "wss://hint.relay.io"));
        assert!(all_relays
            .iter()
            .any(|r| r.as_str() == "wss://received.relay.io"));
    }

    #[tokio::test]
    async fn test_status_tracking() {
        let store = NostrGossipMemory::default();

        let public_key =
            PublicKey::from_hex("68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272")
                .unwrap();

        // Initially should be outdated
        let status = store.get_status(&public_key, GossipListKind::Nip65).await;
        assert!(matches!(status, GossipPublicKeyStatus::Outdated { .. }));

        // Process a NIP-65 event
        let json = r#"{"id":"0a49bed4a1eb0973a68a0d43b7ca62781ffd4e052b91bbadef09e5cf756f6e68","pubkey":"68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272","created_at":1759351841,"kind":10002,"tags":[["alt","Relay list to discover the user's content"],["r","wss://relay.damus.io/"],["r","wss://nostr.wine/"],["r","wss://nostr.oxtr.dev/"],["r","wss://relay.nostr.wirednet.jp/"]],"content":"","sig":"f5bc6c18b0013214588d018c9086358fb76a529aa10867d4d02a75feb239412ae1c94ac7c7917f6e6e2303d72f00dc4e9b03b168ef98f3c3c0dec9a457ce0304"}"#;
        let event = Event::from_json(json).unwrap();
        store.process(&event, None).await.unwrap();

        // Update fetch attempt
        store
            ._update_fetch_attempt(&public_key, GossipListKind::Nip65)
            .await;

        // Should now be updated
        let status = store.get_status(&public_key, GossipListKind::Nip65).await;
        assert!(matches!(status, GossipPublicKeyStatus::Updated));
    }

    #[tokio::test]
    async fn test_empty_results() {
        let store = NostrGossipMemory::default();

        // Random public key with no data
        let public_key =
            PublicKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        // Should return empty set
        let relays = store
            ._get_best_relays(&public_key, BestRelaySelection::Read { limit: 10 })
            .await;

        assert_eq!(relays.len(), 0);
    }
}
