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
