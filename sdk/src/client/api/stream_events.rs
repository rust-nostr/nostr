use std::collections::HashMap;
use std::future::IntoFuture;
use std::time::Duration;

use nostr::types::url::RelayUrl;
use nostr::{Event, Filter};

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
    timeout: Option<Duration>,
    policy: ReqExitPolicy,
}

impl<'client, 'url> StreamEvents<'client, 'url> {
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
                .stream_events(targets, self.timeout, self.policy)
                .await?)
        })
    }
}
