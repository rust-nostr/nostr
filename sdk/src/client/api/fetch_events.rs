use std::collections::{HashMap, HashSet};
use std::future::IntoFuture;
use std::time::Duration;

use futures::StreamExt;
use nostr::{PublicKey, RelayUrl};
use nostr_database::Events;

use super::req_target::ReqTarget;
use super::util::build_targets;
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
            // Build targets (decompose filters via gossip if available)
            let targets = build_targets(self.client, self.target).await?;

            // Extract relay → pubkeys mapping from decomposed filters for delivery tracking
            let relay_pubkeys: HashMap<RelayUrl, HashSet<PublicKey>> = targets
                .iter()
                .map(|(url, filters)| {
                    let pks: HashSet<PublicKey> = filters
                        .iter()
                        .filter_map(|f| f.authors.as_ref())
                        .flatten()
                        .copied()
                        .collect();
                    (url.clone(), pks)
                })
                .filter(|(_, pks)| !pks.is_empty())
                .collect();

            // Stream events from pool
            let mut stream = self
                .client
                .pool()
                .stream_events(targets, None, self.timeout, self.policy)
                .await?;

            let mut events: Events = Events::default();

            // Track which (relay, pubkey) pairs actually delivered
            let mut delivered: HashSet<(RelayUrl, PublicKey)> = HashSet::new();

            // Collect events
            while let Some((url, result)) = stream.next().await {
                // NOTE: not propagate the error here! A single error by any of the relays would stop the entire fetching process.
                match result {
                    Ok(event) => {
                        // Track delivery: this relay delivered an event from this author
                        if relay_pubkeys.contains_key(&url) {
                            delivered.insert((url, event.pubkey));
                        }

                        // To find out more about why the `force_insert` was used, search for EVENTS_FORCE_INSERT in the code.
                        events.force_insert(event);
                    }
                    Err(e) => {
                        tracing::error!(url = %url, error = %e, "Failed to handle streamed event");
                    }
                }
            }

            // Record delivery stats to gossip store (best-effort, errors are ignored)
            if let Some(gossip) = self.client.gossip() {
                let store = gossip.store();
                for (url, pubkeys) in &relay_pubkeys {
                    for pk in pubkeys {
                        let d = if delivered.contains(&(url.clone(), *pk)) {
                            1
                        } else {
                            0
                        };
                        let _ = store.record_delivery(url, pk, d, 1).await;
                    }
                }
            }

            Ok(events)
        })
    }
}
