use std::future::IntoFuture;
use std::time::Duration;

use futures::StreamExt;
use nostr_database::Events;

use super::req_target::ReqTarget;
use super::stream_events::StreamEvents;
use crate::client::{Client, Error};
use crate::future::BoxedFuture;
use crate::relay::ReqExitPolicy;

/// Fetch events
#[must_use = "Does nothing unless you await!"]
pub struct FetchEvents<'client, 'url> {
    // --------------------------------------------------
    // WHEN ADDING NEW OPTIONS HERE,
    // REMEMBER TO UPDATE THE "Configuration" SECTION in
    // Client::fetch_events DOC.
    // --------------------------------------------------
    client: &'client Client,
    target: ReqTarget<'url>,
    timeout: Option<Duration>,
    policy: ReqExitPolicy,
}

impl<'client, 'url> FetchEvents<'client, 'url> {
    pub(crate) fn new(client: &'client Client, target: ReqTarget<'url>) -> Self {
        Self {
            client,
            target,
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

impl<'client, 'url> IntoFuture for FetchEvents<'client, 'url>
where
    'url: 'client,
{
    type Output = Result<Events, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Stream events
            let mut stream: StreamEvents<'client, 'url> =
                self.client.stream_events(self.target).policy(self.policy);

            // Set timeout
            if let Some(timeout) = self.timeout {
                stream = stream.timeout(timeout);
            }

            // Execute stream
            let mut stream = stream.await?;

            let mut events: Events = Events::default();

            // Collect events
            while let Some((url, result)) = stream.next().await {
                // NOTE: not propagate the error here! A single error by any of the relays would stop the entire fetching process.
                match result {
                    Ok(event) => {
                        // To find out more about why the `force_insert` was used, search for EVENTS_FORCE_INSERT in the code.
                        events.force_insert(event);
                    }
                    Err(e) => {
                        tracing::error!(url = %url, error = %e, "Failed to handle streamed event");
                    }
                }
            }

            Ok(events)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nostr::event::FinalizeEvent;
    use nostr::{EventBuilder, Filter, Keys, Kind};

    use crate::authenticator::SignerAuthenticator;
    use crate::test_utils::{
        setup_client, setup_client_with_authenticator, setup_nip42_read_local_relay,
    };

    #[tokio::test]
    async fn test_client_fetch_events_dont_resubscribes_after_auth_required_closed_without_authenticator()
     {
        let local = setup_nip42_read_local_relay().await;

        let keys = Keys::generate();
        let expected = EventBuilder::text_note("Test").finalize(&keys).unwrap();
        local.add_event(expected.clone()).await.unwrap();

        let client = setup_client(local.url().await).await;

        let filter = Filter::new().kind(Kind::TextNote).limit(1);

        let events = client
            .fetch_events(filter)
            .timeout(Duration::from_secs(5))
            .await
            .unwrap();

        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_client_fetch_events_resubscribes_after_auth_required_closed() {
        let local = setup_nip42_read_local_relay().await;

        let keys = Keys::generate();
        let expected = EventBuilder::text_note("Test").finalize(&keys).unwrap();
        local.add_event(expected.clone()).await.unwrap();

        let authenticator = SignerAuthenticator::new(keys);
        let client = setup_client_with_authenticator(local.url().await, authenticator).await;

        let filter = Filter::new().kind(Kind::TextNote).limit(1);

        let events = client
            .fetch_events(filter)
            .timeout(Duration::from_secs(5))
            .await
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events.first().map(|event| event.id), Some(expected.id));
    }
}
