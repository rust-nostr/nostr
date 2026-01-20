use std::collections::{HashMap, HashSet};
use std::future::{Future, IntoFuture};
use std::pin::Pin;

use nostr::{EventId, Filter, RelayUrl, RelayUrlArg, Timestamp};

use super::blocking::Blocking;
use super::output::Output;
use crate::client::{Client, Error};
use crate::relay::{Reconciliation, RelayCapabilities, SyncOptions};

/// Sync events
///
/// <https://github.com/nostr-protocol/nips/blob/master/77.md>
#[must_use = "Does nothing unless you await!"]
pub struct SyncEvents<'client, 'url> {
    client: &'client Client,
    filter: Filter,
    with: Vec<RelayUrlArg<'url>>,
    opts: SyncOptions,
}

impl<'client, 'url> SyncEvents<'client, 'url> {
    #[inline]
    pub(crate) fn new(client: &'client Client, filter: Filter) -> Self {
        Self {
            client,
            filter,
            with: Vec::new(),
            opts: SyncOptions::new(),
        }
    }

    /// Set relays to sync with
    pub fn with<I, U>(mut self, relays: I) -> Self
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'url>>,
    {
        self.with.extend(relays.into_iter().map(Into::into));
        self
    }

    /// Set sync options
    #[inline]
    pub fn opts(mut self, opts: SyncOptions) -> Self {
        self.opts = opts;
        self
    }

    async fn exec(self) -> Result<Output<Reconciliation>, Error> {
        // Build targets
        let targets: HashMap<RelayUrl, (Filter, Vec<(EventId, Timestamp)>)> =
            match (&self.client.gossip, self.with.is_empty()) {
                // Gossip is available, and there are no specified relays: use gossip
                (Some(gossip), true) => {
                    // Break down filter
                    let filters: HashMap<RelayUrl, Filter> =
                        self.client.break_down_filter(gossip, self.filter).await?;

                    // Make targets
                    make_sync_targets(self.client, filters).await?
                }
                // There are specified relays: use them as targets
                (_, false) => {
                    // Construct filters
                    let filters: HashMap<RelayUrl, Filter> =
                        construct_filters(self.with, self.filter)?;

                    // Make targets
                    make_sync_targets(self.client, filters).await?
                }
                // Gossip is not available, and there are no specified targets: use all relays as targets
                (None, true) => {
                    // Get all READ and WRITE relays from pool
                    let urls: HashSet<RelayUrl> = self
                        .client
                        .pool
                        .relay_urls_with_any_cap(RelayCapabilities::READ | RelayCapabilities::WRITE)
                        .await;

                    // Construct filters
                    let filters: HashMap<RelayUrl, Filter> = construct_filters(urls, self.filter)?;

                    // Make targets
                    make_sync_targets(self.client, filters).await?
                }
            };

        Ok(self.client.pool.sync(targets, self.opts).await?)
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
    type Output = Result<Output<Reconciliation>, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl<'client, 'url> Blocking for SyncEvents<'client, 'url> where 'url: 'client {}
