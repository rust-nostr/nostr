use std::collections::HashMap;
use std::future::IntoFuture;
use std::time::Duration;

use nostr::types::url::RelayUrl;
use nostr::{Event, Filter, SubscriptionId};

use super::req_target::ReqTarget;
use super::util::build_targets;
use crate::client::{Client, Error};
use crate::future::BoxedFuture;
use crate::relay::{self, ReqExitPolicy};
use crate::stream::BoxedStream;

type EventStream = BoxedStream<(RelayUrl, Result<Event, relay::Error>)>;

/// Stream events
#[must_use = "Does nothing unless you await!"]
pub struct StreamEvents<'client, 'url> {
    // --------------------------------------------------
    // WHEN ADDING NEW OPTIONS HERE,
    // REMEMBER TO UPDATE THE "Configuration" SECTION in
    // Client::stream_events DOC.
    // --------------------------------------------------
    client: &'client Client,
    target: ReqTarget<'url>,
    id: Option<SubscriptionId>,
    timeout: Option<Duration>,
    policy: ReqExitPolicy,
}

impl<'client, 'url> StreamEvents<'client, 'url> {
    pub(crate) fn new(client: &'client Client, target: ReqTarget<'url>) -> Self {
        Self {
            client,
            target,
            id: None,
            timeout: None,
            policy: ReqExitPolicy::ExitOnEOSE,
        }
    }

    /// Set a specific subscription ID
    #[inline]
    pub fn with_id(mut self, id: SubscriptionId) -> Self {
        self.id = Some(id);
        self
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

impl<'client, 'url> IntoFuture for StreamEvents<'client, 'url>
where
    'url: 'client,
{
    type Output = Result<EventStream, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Build targets
            let targets: HashMap<RelayUrl, Vec<Filter>> =
                build_targets(self.client, self.target).await?;

            // Stream
            Ok(self
                .client
                .pool()
                .stream_events(targets, self.id, self.timeout, self.policy)
                .await?)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nostr::{Filter, Kind, SubscriptionId};
    use nostr_relay_builder::MockRelay;

    use super::*;

    #[tokio::test]
    async fn test_stream_terminates_on_drop() {
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let client = Client::default();

        client.add_relay(&url).connect().await.unwrap();

        let filter = Filter::new().kind(Kind::TextNote).limit(1);
        let id = SubscriptionId::generate();

        let stream = client
            .stream_events(filter)
            .with_id(id.clone())
            .policy(ReqExitPolicy::WaitForEvents(1))
            .await
            .unwrap();

        let relay = client.relay(&url).await.unwrap().unwrap();

        // Check if relay has the stream subscription
        let exists: bool = relay.subscription(&id).await.is_some();
        assert!(exists);

        // Drop the stream
        // This must terminate the stream and close the subscription
        drop(stream);

        // Wait a bit
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Now the subscription must not exist anymore
        let exists: bool = relay.subscription(&id).await.is_some();
        assert!(!exists);
    }
}
