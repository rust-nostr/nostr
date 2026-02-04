use std::future::IntoFuture;
use std::time::Duration;

use futures::StreamExt;
use nostr::{Event, Filter};
use nostr_database::Events;

use crate::future::BoxedFuture;
use crate::relay::{Error, Relay, ReqExitPolicy};

/// Fetch events
#[must_use = "Does nothing unless you await!"]
pub struct FetchEvents<'relay> {
    relay: &'relay Relay,
    filters: Vec<Filter>,
    timeout: Option<Duration>,
    policy: ReqExitPolicy,
}

impl<'relay> FetchEvents<'relay> {
    pub(crate) fn new(relay: &'relay Relay, filters: Vec<Filter>) -> Self {
        Self {
            relay,
            filters,
            timeout: None,
            policy: ReqExitPolicy::ExitOnEOSE,
        }
    }

    /// Set a timeout
    ///
    /// By default, no timeout is configured.
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set request exit policy (default: [`ReqExitPolicy::ExitOnEOSE`]).
    #[inline]
    pub fn policy(mut self, policy: ReqExitPolicy) -> Self {
        self.policy = policy;
        self
    }
}

impl<'relay> IntoFuture for FetchEvents<'relay> {
    type Output = Result<Events, Error>;
    type IntoFuture = BoxedFuture<'relay, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Construct a new events collection
            let mut events: Events = if self.filters.len() == 1 {
                // SAFETY: this can't panic because the filters are already verified that list isn't empty.
                let filter: &Filter = &self.filters[0];
                Events::new(filter)
            } else {
                // More than a filter, so we can't ensure to respect the limit -> construct a default collection.
                Events::default()
            };

            // Stream events
            let mut stream = self
                .relay
                .stream_events(self.filters)
                .maybe_timeout(self.timeout)
                .policy(self.policy)
                .await?;

            while let Some(res) = stream.next().await {
                // Get event from the result
                let event: Event = res?;

                // Use force insert here!
                // Due to the configurable REQ exit policy, the user may want to wait for events after EOSE.
                // If the filter has a limit, the force insert allows adding events post-EOSE.
                //
                // For example, if we use `Events::insert` here,
                // if the filter is '{"kinds":[1],"limit":3}' and the policy `ReqExitPolicy::WaitForEventsAfterEOSE(1)`,
                // the events collection will discard 1 event because the filter limit is 3 and the total received events are 4.
                //
                // Events::force_insert automatically increases the capacity if needed, without discarding events.
                //
                // LOOKUP_ID: EVENTS_FORCE_INSERT
                events.force_insert(event);
            }

            Ok(events)
        })
    }
}

#[cfg(test)]
mod tests {
    use nostr::message::MachineReadablePrefix;
    use nostr::{EventBuilder, Keys, Kind, Metadata};
    use nostr_relay_builder::prelude::*;

    use super::*;
    use crate::relay::{Error, RelayOptions, RelayStatus};

    /// Setup public (without NIP42 auth) relay with N events to test event fetching
    ///
    /// **Adds ONLY text notes**
    async fn setup_event_fetching_relay(num_events: usize) -> (Relay, MockRelay) {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let relay = Relay::new(url);
        relay.connect();

        // Signer
        let keys = Keys::generate();

        // Send some events
        for i in 0..num_events {
            let event = EventBuilder::text_note(i.to_string())
                .sign_with_keys(&keys)
                .unwrap();
            relay.send_event(&event).await.unwrap();
        }

        (relay, mock)
    }

    #[tokio::test]
    async fn test_fetch_events_ban_relay() {
        // Mock relay
        let opts = LocalRelayTestOptions {
            unresponsive_connection: None,
            send_random_events: true,
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = mock.url().await;

        let relay = Relay::builder(url)
            .opts(
                RelayOptions::default()
                    .verify_subscriptions(true)
                    .ban_relay_on_mismatch(true),
            )
            .build();

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay
            .try_connect()
            .timeout(Duration::from_secs(3))
            .await
            .unwrap();

        assert_eq!(relay.status(), RelayStatus::Connected);

        let filter = Filter::new().kind(Kind::Metadata);
        relay
            .fetch_events(filter)
            .timeout(Duration::from_secs(3))
            .await
            .unwrap();

        assert_eq!(relay.status(), RelayStatus::Banned);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_nip42_fetch_events_without_signer() {
        // Mock relay
        let opts = LocalRelayBuilderNip42 {
            mode: LocalRelayBuilderNip42Mode::Read,
        };
        let mock = LocalRelay::builder().nip42(opts).build();
        mock.run().await.unwrap();
        let url = mock.url().await;

        let relay: Relay = Relay::new(url);

        relay.connect();

        // Signer
        let keys = Keys::generate();

        // Send an event
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        relay.send_event(&event).await.unwrap();

        let filter = Filter::new().kind(Kind::TextNote).limit(3);

        // Disable NIP42 auto auth
        relay.inner.state.automatic_authentication(false);

        // Unauthenticated fetch (MUST return error)
        let err = relay
            .fetch_events(filter.clone())
            .timeout(Duration::from_secs(5))
            .await
            .unwrap_err();
        match err {
            Error::RelayMessage(msg) => {
                assert_eq!(
                    MachineReadablePrefix::parse(&msg).unwrap(),
                    MachineReadablePrefix::AuthRequired
                );
            }
            e => panic!("Unexpected error: {e}"),
        }

        // Enable NIP42 auto auth
        relay.inner.state.automatic_authentication(true);

        // Unauthenticated fetch (MUST return error)
        let err = relay
            .fetch_events(filter.clone())
            .timeout(Duration::from_secs(5))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::AuthenticationFailed));
    }

    #[tokio::test]
    async fn test_nip42_fetch_events_with_signer() {
        // Mock relay
        let opts = LocalRelayBuilderNip42 {
            mode: LocalRelayBuilderNip42Mode::Read,
        };
        let mock = LocalRelay::builder().nip42(opts).build();
        mock.run().await.unwrap();
        let url = mock.url().await;

        // Signer
        let keys = Keys::generate();

        let relay: Relay = Relay::builder(url).signer(keys.clone()).build();

        relay.connect();

        // Send an event
        let event = EventBuilder::text_note("Test")
            .sign_with_keys(&keys)
            .unwrap();
        relay.send_event(&event).await.unwrap();

        let filter = Filter::new().kind(Kind::TextNote).limit(3);

        // Disable NIP42 auto auth
        relay.inner.state.automatic_authentication(false);

        // NIP-42 auth disabled, so it's an unauthenticated REQ (MUST return error)
        let err = relay
            .fetch_events(filter.clone())
            .timeout(Duration::from_secs(5))
            .await
            .unwrap_err();
        match err {
            Error::RelayMessage(msg) => {
                assert_eq!(
                    MachineReadablePrefix::parse(&msg).unwrap(),
                    MachineReadablePrefix::AuthRequired
                );
            }
            e => panic!("Unexpected error: {e}"),
        }

        // Enable NIP42 auto auth
        relay.inner.state.automatic_authentication(true);

        // Authenticated fetch
        let res = relay
            .fetch_events(filter)
            .timeout(Duration::from_secs(5))
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_events_exit_on_eose() {
        let (relay, _mock) = setup_event_fetching_relay(5).await;

        // Exit on EOSE
        let events = relay
            .fetch_events(Filter::new().kind(Kind::TextNote))
            .timeout(Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(events.len(), 5);

        // Exit on EOSE
        let events = relay
            .fetch_events(Filter::new().kind(Kind::TextNote).limit(3))
            .timeout(Duration::from_secs(5))
            .policy(ReqExitPolicy::ExitOnEOSE)
            .await
            .unwrap();
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn test_fetch_events_wait_for_events() {
        let (relay, _mock) = setup_event_fetching_relay(5).await;

        let events = relay
            .fetch_events(Filter::new().kind(Kind::TextNote))
            .timeout(Duration::from_secs(15))
            .policy(ReqExitPolicy::WaitForEvents(2))
            .await
            .unwrap();
        assert_eq!(events.len(), 2); // Requested all text notes but exit after receive 2

        // Task to send additional event
        let r = relay.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Signer
            let keys = Keys::generate();

            // Build and send event
            let event = EventBuilder::metadata(&Metadata::new().name("Test"))
                .sign_with_keys(&keys)
                .unwrap();
            r.send_event(&event).await.unwrap();
        });

        let events = relay
            .fetch_events(Filter::new().kind(Kind::Metadata))
            .timeout(Duration::from_secs(5))
            .policy(ReqExitPolicy::WaitForEvents(1))
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_fetch_events_wait_for_events_after_eose() {
        let (relay, _mock) = setup_event_fetching_relay(10).await;

        // Task to send additional events
        let r = relay.clone();
        tokio::spawn(async move {
            // Signer
            let keys = Keys::generate();

            // Send more events
            for _ in 0..2 {
                // Sleep
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Build and send event
                let event = EventBuilder::text_note("Additional")
                    .sign_with_keys(&keys)
                    .unwrap();
                r.send_event(&event).await.unwrap();
            }
        });

        let events = relay
            .fetch_events(Filter::new().kind(Kind::TextNote).limit(3))
            .timeout(Duration::from_secs(15))
            .policy(ReqExitPolicy::WaitForEventsAfterEOSE(2))
            .await
            .unwrap();
        assert_eq!(events.len(), 5); // 3 events received until EOSE + 2 new events
    }

    #[tokio::test]
    async fn test_fetch_events_wait_for_duration_after_eose() {
        let (relay, _mock) = setup_event_fetching_relay(5).await;

        // Task to send additional events
        let r = relay.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Signer
            let keys = Keys::generate();

            // Send more events
            for _ in 0..2 {
                // Build and send event
                let event = EventBuilder::text_note("Additional")
                    .sign_with_keys(&keys)
                    .unwrap();
                r.send_event(&event).await.unwrap();

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        let events = relay
            .fetch_events(Filter::new().kind(Kind::TextNote))
            .timeout(Duration::from_secs(15))
            .policy(ReqExitPolicy::WaitDurationAfterEOSE(Duration::from_secs(3)))
            .await
            .unwrap();
        assert_eq!(events.len(), 6); // 5 events received until EOSE + 1 new events
    }
}
