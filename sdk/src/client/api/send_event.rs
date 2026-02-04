use std::collections::HashSet;
use std::future::IntoFuture;
use std::iter;
use std::time::Duration;

use nostr::{Event, EventId, Kind, RelayUrl, RelayUrlArg};
use nostr_gossip::{BestRelaySelection, GossipListKind};

use super::output::Output;
use crate::client::gossip::GossipWrapper;
use crate::client::{Client, Error};
use crate::future::BoxedFuture;
use crate::relay::RelayCapabilities;

enum OverwritePolicy<'url> {
    // All WRITE relays
    Broadcast,
    // To specific relays
    To(Vec<RelayUrlArg<'url>>),
    // To NIP-17 relays
    ToNip17,
    // To NIP-65 relays
    ToNip65,
}

/// Send event
#[must_use = "Does nothing unless you await!"]
pub struct SendEvent<'client, 'event, 'url> {
    // --------------------------------------------------
    // WHEN ADDING NEW OPTIONS HERE,
    // REMEMBER TO UPDATE THE "Configuration" SECTION in
    // Client::send_event DOC.
    // --------------------------------------------------
    client: &'client Client,
    event: &'event Event,
    policy: Option<OverwritePolicy<'url>>,
    save_into_database: bool,
    wait_for_ok_timeout: Duration,
    wait_for_authentication_timeout: Duration,
}

impl<'client, 'event, 'url> SendEvent<'client, 'event, 'url> {
    pub(crate) fn new(client: &'client Client, event: &'event Event) -> Self {
        Self {
            client,
            event,
            policy: None,
            save_into_database: true,
            wait_for_ok_timeout: Duration::from_secs(10),
            wait_for_authentication_timeout: Duration::from_secs(10),
        }
    }

    /// Send event to all relays with [`RelayCapabilities::WRITE`] capability.
    ///
    /// This overwrites the following methods:
    /// - [`SendEvent::to`]
    /// - [`SendEvent::to_nip17`]
    /// - [`SendEvent::to_nip65`]
    ///
    /// [`RelayCapabilities::WRITE`]: crate::relay::RelayCapabilities::WRITE
    #[inline]
    pub fn broadcast(mut self) -> Self {
        self.policy = Some(OverwritePolicy::Broadcast);
        self
    }

    /// Send event to specific relays
    ///
    /// This overwrites the following methods:
    /// - [`SendEvent::broadcast`]
    /// - [`SendEvent::to_nip17`]
    /// - [`SendEvent::to_nip65`]
    pub fn to<I, T>(mut self, urls: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<RelayUrlArg<'url>>,
    {
        self.policy = Some(OverwritePolicy::To(
            urls.into_iter().map(Into::into).collect(),
        ));
        self
    }

    /// Send event to NIP-17 relays
    ///
    /// This overwrites the following methods:
    /// - [`SendEvent::to`]
    /// - [`SendEvent::broadcast`]
    /// - [`SendEvent::to_nip65`]
    ///
    /// Returns [`Error::GossipNotConfigured`] if gossip is not configured.
    #[inline]
    pub fn to_nip17(mut self) -> Self {
        self.policy = Some(OverwritePolicy::ToNip17);
        self
    }

    /// Send event to NIP-65 relays
    ///
    /// This overwrites the following methods:
    /// - [`SendEvent::to`]
    /// - [`SendEvent::broadcast`]
    /// - [`SendEvent::to_nip17`]
    ///
    /// Returns [`Error::GossipNotConfigured`] if gossip is not configured.
    #[inline]
    pub fn to_nip65(mut self) -> Self {
        self.policy = Some(OverwritePolicy::ToNip65);
        self
    }

    /// Save the event into the database (default: true)
    ///
    /// If `true`, the event is immediately saved into the database.
    #[inline]
    pub fn save_into_database(mut self, enabled: bool) -> Self {
        self.save_into_database = enabled;
        self
    }

    /// Timeout for waiting for the `OK` message from relay (default: 10 sec)
    #[inline]
    pub fn ok_timeout(mut self, timeout: Duration) -> Self {
        self.wait_for_ok_timeout = timeout;
        self
    }

    /// Timeout for waiting that relay authenticates (default: 10 sec)
    #[inline]
    pub fn authentication_timeout(mut self, timeout: Duration) -> Self {
        self.wait_for_authentication_timeout = timeout;
        self
    }
}

async fn gossip_send_event(
    client: &Client,
    gossip: &GossipWrapper,
    event: &Event,
    is_nip17: bool,
    wait_for_ok_timeout: Duration,
    wait_for_authentication_timeout: Duration,
) -> Result<Output<EventId>, Error> {
    let is_contact_list: bool = event.kind == Kind::ContactList;
    let is_gift_wrap: bool = event.kind == Kind::GiftWrap;

    // Get involved public keys and check what are up to date in the gossip graph and which ones require an update.
    if is_gift_wrap {
        let kind: GossipListKind = if is_nip17 {
            GossipListKind::Nip17
        } else {
            GossipListKind::Nip65
        };

        // Get only p tags since the author of a gift wrap is randomized
        let public_keys = event.tags.public_keys().copied();
        client
            .check_and_update_gossip(gossip, public_keys, kind)
            .await?;
    } else if is_contact_list {
        // Contact list, update only author
        client
            .check_and_update_gossip(gossip, [event.pubkey], GossipListKind::Nip65)
            .await?;
    } else {
        // Get all public keys involved in the event: author + p tags
        let public_keys = event
            .tags
            .public_keys()
            .copied()
            .chain(iter::once(event.pubkey));
        client
            .check_and_update_gossip(gossip, public_keys, GossipListKind::Nip65)
            .await?;
    };

    // Check if NIP17 or NIP65
    let urls: HashSet<RelayUrl> = if is_nip17 && is_gift_wrap {
        // Get NIP17 relays
        // Get only for relays for p tags since gift wraps are signed with random key (random author)
        let relays = gossip
            .get_relays(
                event.tags.public_keys(),
                BestRelaySelection::PrivateMessage { limit: 3 },
                client.config.gossip_allowed,
            )
            .await?;

        // Clients SHOULD publish kind 14 events to the 10050-listed relays.
        // If that is not found, that indicates the user is not ready to receive messages under this NIP and clients shouldn't try.
        //
        // <https://github.com/nostr-protocol/nips/blob/6e7a618e7f873bb91e743caacc3b09edab7796a0/17.md>
        if relays.is_empty() {
            return Err(Error::PrivateMsgRelaysNotFound);
        }

        // Add outbox and inbox relays
        for url in relays.iter().cloned() {
            client
                .add_relay(url)
                .capabilities(RelayCapabilities::GOSSIP)
                .and_connect()
                .await?;
        }

        relays
    } else {
        // Get OUTBOX, HINTS and MOST_RECEIVED relays for the author
        let mut relays: HashSet<RelayUrl> = gossip
            .get_best_relays(
                &event.pubkey,
                BestRelaySelection::All {
                    read: 0, // No read relays
                    write: client.config.gossip_limits.write_relays_per_user,
                    hints: client.config.gossip_limits.hint_relays_per_user,
                    most_received: client.config.gossip_limits.most_used_relays_per_user,
                },
                client.config.gossip_allowed,
            )
            .await?;

        // Extend with INBOX, HINTS and MOST_RECEIVED relays for the tags
        if !is_contact_list {
            let inbox_hints_most_recv: HashSet<RelayUrl> = gossip
                .get_relays(
                    event.tags.public_keys(),
                    BestRelaySelection::All {
                        read: client.config.gossip_limits.read_relays_per_user,
                        write: 0, // No write relays
                        hints: client.config.gossip_limits.hint_relays_per_user,
                        most_received: client.config.gossip_limits.most_used_relays_per_user,
                    },
                    client.config.gossip_allowed,
                )
                .await?;

            relays.extend(inbox_hints_most_recv);
        }

        // Add OUTBOX and INBOX relays
        for url in relays.iter().cloned() {
            client
                .add_relay(url)
                .capabilities(RelayCapabilities::GOSSIP)
                .and_connect()
                .await?;
        }

        // Get WRITE relays
        let write_relays: HashSet<RelayUrl> = client.pool.write_relay_urls().await;

        // Extend relays with WRITE ones
        relays.extend(write_relays);

        // Return all relays
        relays
    };

    // Send event
    Ok(client
        .pool
        .send_event(
            urls,
            event,
            wait_for_ok_timeout,
            wait_for_authentication_timeout,
        )
        .await?)
}

impl<'client, 'event, 'url> IntoFuture for SendEvent<'client, 'event, 'url>
where
    'event: 'client,
    'url: 'client,
{
    type Output = Result<Output<EventId>, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Save event into database
            if self.save_into_database {
                self.client.database().save_event(self.event).await?;
            }

            // Process event for gossip, independently of the policy
            if let Some(gossip) = &self.client.gossip {
                gossip.process(self.event, None).await?;
            }

            match (self.policy, &self.client.gossip) {
                // No overwrite policy or send to NIP-65 and gossip available: send to NIP-65 relays
                (None | Some(OverwritePolicy::ToNip65), Some(gossip)) => {
                    gossip_send_event(
                        self.client,
                        gossip,
                        self.event,
                        false,
                        self.wait_for_ok_timeout,
                        self.wait_for_authentication_timeout,
                    )
                    .await
                }
                // Send to NIP-17 and gossip available: send to NIP-17 relays
                (Some(OverwritePolicy::ToNip17), Some(gossip)) => {
                    gossip_send_event(
                        self.client,
                        gossip,
                        self.event,
                        true,
                        self.wait_for_ok_timeout,
                        self.wait_for_authentication_timeout,
                    )
                    .await
                }
                // Send to gossip, but gossip is not available: error
                (Some(OverwritePolicy::ToNip17 | OverwritePolicy::ToNip65), None) => {
                    Err(Error::GossipNotConfigured)
                }
                // Send to specific relays
                (Some(OverwritePolicy::To(list)), _) => {
                    let mut urls: HashSet<RelayUrl> = HashSet::with_capacity(list.len());

                    for url in list {
                        urls.insert(url.try_into_relay_url()?.into_owned());
                    }

                    Ok(self
                        .client
                        .pool
                        .send_event(
                            urls,
                            self.event,
                            self.wait_for_ok_timeout,
                            self.wait_for_authentication_timeout,
                        )
                        .await?)
                }
                // Send to all WRITE relays
                (Some(OverwritePolicy::Broadcast), _) => {
                    let urls: HashSet<RelayUrl> = self.client.pool.write_relay_urls().await;
                    Ok(self
                        .client
                        .pool
                        .send_event(
                            urls,
                            self.event,
                            self.wait_for_ok_timeout,
                            self.wait_for_authentication_timeout,
                        )
                        .await?)
                }
                // No overwrite policy and no gossip available: send to all WRITE relays
                (None, None) => {
                    let urls: HashSet<RelayUrl> = self.client.pool.write_relay_urls().await;
                    Ok(self
                        .client
                        .pool
                        .send_event(
                            urls,
                            self.event,
                            self.wait_for_ok_timeout,
                            self.wait_for_authentication_timeout,
                        )
                        .await?)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use nostr::prelude::*;
    use nostr_gossip::GossipAllowedRelays;
    use nostr_gossip_memory::store::NostrGossipMemory;
    use nostr_relay_builder::MockRelay;

    use super::*;
    use crate::client::GossipRelayLimits;

    #[tokio::test]
    async fn test_send_event() {
        let mock1 = MockRelay::run().await.unwrap();
        let url1 = mock1.url().await;
        let mock2 = MockRelay::run().await.unwrap();
        let url2 = mock2.url().await;
        let mock3 = MockRelay::run().await.unwrap();
        let url3 = mock3.url().await;

        let client: Client = Client::default();

        // Add 2 READ and WRITE relays
        client.add_relay(&url1).await.unwrap();
        client.add_relay(&url2).await.unwrap();

        // Add a READ-only relay
        client
            .add_relay(&url3)
            .capabilities(RelayCapabilities::READ)
            .await
            .unwrap();

        client.connect().await;

        let keys = Keys::generate();
        let event = EventBuilder::text_note("Broadcast test")
            .sign_with_keys(&keys)
            .unwrap();

        // Send event (broadcast to all WRITE relays by default)
        let output = client.send_event(&event).await.unwrap();

        assert_eq!(output.success.len(), 2);
        assert!(output.success.contains(&url1));
        assert!(output.success.contains(&url2));
        assert!(!output.success.contains(&url3));
        assert!(output.failed.is_empty());
        assert_eq!(output.val, event.id);
    }

    #[tokio::test]
    async fn test_send_event_to() {
        let mock1 = MockRelay::run().await.unwrap();
        let url1 = mock1.url().await;
        let mock2 = MockRelay::run().await.unwrap();
        let url2 = mock2.url().await;

        let client = Client::default();
        client.add_relay(&url1).await.unwrap();
        client.add_relay(&url2).await.unwrap();
        client.connect().await;

        let keys = Keys::generate();
        let event = EventBuilder::text_note("Targeted test")
            .sign_with_keys(&keys)
            .unwrap();

        // Send only to relay 1
        let output = client.send_event(&event).to([&url1]).await.unwrap();

        assert_eq!(output.success.len(), 1);
        assert!(output.success.contains(&url1));
        assert!(!output.success.contains(&url2));
        assert!(output.failed.is_empty());
        assert_eq!(output.val, event.id);
    }

    #[tokio::test]
    async fn test_send_event_broadcast() {
        let mock1 = MockRelay::run().await.unwrap();
        let url1 = mock1.url().await;
        let mock2 = MockRelay::run().await.unwrap();
        let url2 = mock2.url().await;
        let mock3 = MockRelay::run().await.unwrap();
        let url3 = mock3.url().await;

        // Configure client with gossip
        let gossip: NostrGossipMemory = NostrGossipMemory::unbounded();
        let client: Client = Client::builder().gossip(gossip).build();

        // Add 2 READ and WRITE relays
        client.add_relay(&url1).await.unwrap();
        client.add_relay(&url2).await.unwrap();

        // Add a READ-only relay
        client
            .add_relay(&url3)
            .capabilities(RelayCapabilities::READ)
            .await
            .unwrap();

        client.connect().await;

        let keys = Keys::generate();
        let event = EventBuilder::text_note("Force to all test")
            .sign_with_keys(&keys)
            .unwrap();

        // Force send to all WRITE instead of using gossip
        let output = client.send_event(&event).broadcast().await.unwrap();

        assert_eq!(output.success.len(), 2);
        assert!(output.success.contains(&url1));
        assert!(output.success.contains(&url2));
        assert!(!output.success.contains(&url3));
        assert!(output.failed.is_empty());
        assert_eq!(output.val, event.id);
    }

    #[tokio::test]
    async fn test_send_event_with_auto_gossip() {
        // Setup Outbox Relay (where the user wants to receive/send events)
        let outbox_mock = MockRelay::run().await.unwrap();
        let outbox_url = outbox_mock.url().await;

        // Setup Discovery Relay (where NIP-65 lists are stored)
        let discovery_mock = MockRelay::run().await.unwrap();
        let discovery_url = discovery_mock.url().await;

        // Setup a generic "Public" Relay
        let public_mock = MockRelay::run().await.unwrap();
        let public_url = public_mock.url().await;

        // Setup User A keys and their Relay List (NIP-65) pointing to the Outbox Relay
        let keys_a = Keys::generate();
        let relay_list = EventBuilder::relay_list([(outbox_url.clone(), None)])
            .sign_with_keys(&keys_a)
            .unwrap();
        let res = discovery_mock.add_event(relay_list).await.unwrap();
        assert!(res.is_success());

        // Configure Client with Gossip
        let gossip = NostrGossipMemory::unbounded();
        let client = Client::builder()
            .gossip(gossip)
            .gossip_limits(GossipRelayLimits {
                read_relays_per_user: 2,
                write_relays_per_user: 2,
                hint_relays_per_user: 1,
                most_used_relays_per_user: 0, // Disable the most used, as it would be the discovery one
                nip17_relays: 3,
            })
            .gossip_allowed(GossipAllowedRelays {
                onion: true,
                local: true,
                without_tls: true,
            })
            .build();

        // The client only knows about the Discovery and Public relays initially
        client
            .add_relay(&discovery_url)
            .capabilities(RelayCapabilities::DISCOVERY)
            .await
            .unwrap();
        client.add_relay(&public_url).await.unwrap();
        client.connect().await;

        // Verify that the client doesn't have the outbox relay
        assert!(client.relay(&outbox_url).await.unwrap().is_none());

        // Verify capabilities
        let relay = client.relay(&discovery_url).await.unwrap().unwrap();
        assert_eq!(relay.capabilities().load(), RelayCapabilities::DISCOVERY);

        // Now, send a Text Note from User A.
        // The gossip engine should:
        // - See the author is User A
        // - Fetch User A's relay list from Discovery/Public relays (or local cache)
        // - Identify 'outbox_url' as the target
        // - Automatically connect to 'outbox_url'
        // - Send the event to the outbox and public relay
        let event = EventBuilder::text_note("Gossip test")
            .sign_with_keys(&keys_a)
            .unwrap();

        // Send event using default config (must be sent to gossip)
        let output = client.send_event(&event).await.unwrap();

        // Verify output
        assert_eq!(output.success.len(), 2);
        assert!(output.success.contains(&outbox_url));
        assert!(output.success.contains(&public_url));
        assert!(!output.success.contains(&discovery_url));
        assert!(output.failed.is_empty());
        assert_eq!(output.val, event.id);

        // Verify the client now has the outbox relay in its pool with GOSSIP capability
        let outbox_relay = client.relay(&outbox_url).await.unwrap().unwrap();
        assert_eq!(
            outbox_relay.capabilities().load(),
            RelayCapabilities::GOSSIP
        );
    }

    #[tokio::test]
    async fn test_send_event_to_nip65_without_gossip() {
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let client: Client = Client::default();
        client.add_relay(&url).await.unwrap();
        client.connect().await;

        let keys = Keys::generate();
        let event = EventBuilder::text_note("Broadcast test")
            .sign_with_keys(&keys)
            .unwrap();

        // Send event
        let err = client.send_event(&event).to_nip65().await.unwrap_err();
        assert!(matches!(err, Error::GossipNotConfigured));
    }

    #[tokio::test]
    async fn test_send_event_to_nip17() {
        let inbox_mock = MockRelay::run().await.unwrap();
        let inbox_url = inbox_mock.url().await;

        // Setup Discovery Relay (where NIP-17 lists are stored)
        let discovery_mock = MockRelay::run().await.unwrap();
        let discovery_url = discovery_mock.url().await;

        // Setup a generic "Public" Relay
        let public_mock = MockRelay::run().await.unwrap();
        let public_url = public_mock.url().await;

        // Setup Bob keys and NIP-17 list pointing to the Inbox Relay
        let bob_keys = Keys::generate();
        let relay_list = EventBuilder::nip17_relay_list([inbox_url.clone()])
            .sign_with_keys(&bob_keys)
            .unwrap();
        let res = discovery_mock.add_event(relay_list).await.unwrap();
        assert!(res.is_success());

        // Configure Client with Gossip
        let gossip = NostrGossipMemory::unbounded();
        let client = Client::builder()
            .gossip(gossip)
            .gossip_allowed(GossipAllowedRelays {
                onion: true,
                local: true,
                without_tls: true,
            })
            .build();

        // The client only knows about the Discovery and Public relays initially
        client
            .add_relay(&discovery_url)
            .capabilities(RelayCapabilities::DISCOVERY)
            .await
            .unwrap();
        client.add_relay(&public_url).await.unwrap();
        client.connect().await;

        // Verify that the client doesn't have the inbox relay
        assert!(client.relay(&inbox_url).await.unwrap().is_none());

        // Sends an event to the inbox relay
        // NOTE: this is not a NIP-17 event, as the nip59 feature is required, so we are sending a fake gift wrap tagging the recipient
        let event = EventBuilder::new(Kind::GiftWrap, "payload")
            .tag(Tag::public_key(bob_keys.public_key))
            .sign_with_keys(&Keys::generate())
            .unwrap();
        let output = client.send_event(&event).to_nip17().await.unwrap();

        // Should be sent ONLY to Bob's discovered inbox
        assert_eq!(output.success.len(), 1);
        assert!(output.success.contains(&inbox_url));
        assert!(!output.success.contains(&public_url));
        assert!(!output.success.contains(&discovery_url));
        assert!(output.failed.is_empty());
        assert_eq!(output.val, event.id);

        // Verify the client now has the outbox relay in its pool with GOSSIP capability
        let inbox_relay = client.relay(&inbox_url).await.unwrap().unwrap();
        assert_eq!(inbox_relay.capabilities().load(), RelayCapabilities::GOSSIP);
    }

    #[tokio::test]
    async fn test_send_event_to_nip17_without_gossip() {
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let client: Client = Client::default();
        client.add_relay(&url).await.unwrap();
        client.connect().await;

        // NOTE: this is not a NIP-17 event, as the nip59 feature is required, so we are sending a fake gift wrap tagging the recipient
        let bob_keys = Keys::generate();
        let event = EventBuilder::new(Kind::GiftWrap, "payload")
            .tag(Tag::public_key(bob_keys.public_key))
            .sign_with_keys(&Keys::generate())
            .unwrap();

        // Send event
        let err = client.send_event(&event).to_nip17().await.unwrap_err();
        assert!(matches!(err, Error::GossipNotConfigured));
    }
}
