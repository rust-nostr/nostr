use std::collections::{HashMap, HashSet};
use std::future::IntoFuture;

use nostr::{EventId, Filter, RelayUrl, RelayUrlArg, Timestamp};

use super::output::Output;
use crate::client::{Client, Error};
use crate::future::BoxedFuture;
use crate::relay::{RelayCapabilities, SyncOptions, SyncSummary as RelaySyncSummary};

/// Client negentropy reconciliation summary
///
/// This includes the summary for all relays involved in the reconciliation process.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncSummary {
    /// Events that were stored locally (missing on relay)
    pub local: HashSet<EventId>,
    /// Events that were stored on relay (missing locally)
    pub remote: HashMap<EventId, HashSet<RelayUrl>>,
    /// Events that are **successfully** sent to relays during reconciliation
    pub sent: HashMap<EventId, HashSet<RelayUrl>>,
    /// Event that are **successfully** received from relay during reconciliation
    pub received: HashMap<EventId, HashSet<RelayUrl>>,
    // TODO: should this be HashMap<EventId, HashMap<RelayUrl, String>>?
    /// Send failures
    pub send_failures: HashMap<RelayUrl, HashMap<EventId, String>>,
    // /// Receive failures
    // pub receive: HashMap<RelayUrl, HashMap<EventId, String>>,
}

impl SyncSummary {
    pub(crate) fn merge_relay_summary(&mut self, url: RelayUrl, other: RelaySyncSummary) {
        self.local.extend(other.local);

        // For each remote event, add this relay URL to the set
        for event_id in other.remote {
            self.remote.entry(event_id).or_default().insert(url.clone());
        }

        // For each sent event, add this relay URL to the set
        for event_id in other.sent {
            self.sent.entry(event_id).or_default().insert(url.clone());
        }

        // For each received event, add this relay URL to the set
        for event_id in other.received {
            self.received
                .entry(event_id)
                .or_default()
                .insert(url.clone());
        }

        self.send_failures
            .entry(url)
            .or_default()
            .extend(other.send_failures);

        //self.receive.extend(other.receive);
    }
}

/// Sync events
///
/// <https://github.com/nostr-protocol/nips/blob/master/77.md>
#[must_use = "Does nothing unless you await!"]
pub struct SyncEvents<'client, 'url> {
    client: &'client Client,
    filter: Filter,
    with: Option<Vec<RelayUrlArg<'url>>>,
    opts: SyncOptions,
}

impl<'client, 'url> SyncEvents<'client, 'url> {
    #[inline]
    pub(crate) fn new(client: &'client Client, filter: Filter) -> Self {
        Self {
            client,
            filter,
            with: None,
            opts: SyncOptions::new(),
        }
    }

    // TODO: instead of having this, use an approach like the stream and fetch events?
    /// Set relays to sync with
    pub fn with<I, U>(mut self, relays: I) -> Self
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'url>>,
    {
        let mut list: Vec<RelayUrlArg<'url>> = self.with.unwrap_or_default();
        list.extend(relays.into_iter().map(Into::into));
        self.with = Some(list);
        self
    }

    /// Set sync options
    #[inline]
    pub fn opts(mut self, opts: SyncOptions) -> Self {
        self.opts = opts;
        self
    }
}

fn construct_filters<'url, I, T>(
    urls: I,
    filter: Filter,
) -> Result<HashMap<RelayUrl, Filter>, Error>
where
    I: IntoIterator<Item = T>,
    T: Into<RelayUrlArg<'url>>,
{
    let mut filters: HashMap<RelayUrl, Filter> = HashMap::new();

    for url in urls {
        let url: RelayUrl = url.into().try_into_relay_url()?.into_owned();
        filters.insert(url, filter.clone());
    }

    Ok(filters)
}

async fn make_sync_targets(
    client: &Client,
    filters: HashMap<RelayUrl, Filter>,
) -> Result<HashMap<RelayUrl, (Filter, Vec<(EventId, Timestamp)>)>, Error> {
    let database = client.database();

    let mut f = HashMap::with_capacity(filters.len());

    for (url, filter) in filters.into_iter() {
        // Get negentropy items
        let items: Vec<(EventId, Timestamp)> = database.negentropy_items(filter.clone()).await?;

        f.insert(url, (filter, items));
    }

    Ok(f)
}

impl<'client, 'url> IntoFuture for SyncEvents<'client, 'url>
where
    'url: 'client,
{
    type Output = Result<Output<SyncSummary>, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Build targets
            let targets: HashMap<RelayUrl, (Filter, Vec<(EventId, Timestamp)>)> =
                match (&self.client.gossip, self.with) {
                    // Gossip is available, and there are no specified relays: use gossip
                    (Some(gossip), None) => {
                        // Break down filter
                        let filters: HashMap<RelayUrl, Filter> =
                            self.client.break_down_filter(gossip, self.filter).await?;

                        // Make targets
                        make_sync_targets(self.client, filters).await?
                    }
                    // There are specified relays: use them as targets
                    (_, Some(with)) => {
                        // Construct filters
                        let filters: HashMap<RelayUrl, Filter> =
                            construct_filters(with, self.filter)?;

                        // Make targets
                        make_sync_targets(self.client, filters).await?
                    }
                    // Gossip is not available, and there are no specified targets: use all relays as targets
                    (None, None) => {
                        // Get all READ and WRITE relays from pool
                        let urls: HashSet<RelayUrl> = self
                            .client
                            .pool
                            .relay_urls_with_any_cap(
                                RelayCapabilities::READ | RelayCapabilities::WRITE,
                            )
                            .await;

                        // Construct filters
                        let filters: HashMap<RelayUrl, Filter> =
                            construct_filters(urls, self.filter)?;

                        // Make targets
                        make_sync_targets(self.client, filters).await?
                    }
                };

            Ok(self.client.pool.sync(targets, self.opts).await?)
        })
    }
}

#[cfg(test)]
mod tests {
    use nostr::Kind;

    use super::*;
    use crate::pool;

    #[tokio::test]
    async fn test_sync_with_empty_list_of_relays() {
        let client = Client::default();

        let filter = Filter::default().kind(Kind::TextNote).limit(100);
        let relays: Vec<RelayUrl> = Vec::new();
        let res = client.sync(filter).with(relays).await;

        assert!(matches!(
            res.unwrap_err(),
            Error::RelayPool(pool::Error::NoRelaysSpecified)
        ))
    }
}
