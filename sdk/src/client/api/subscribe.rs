use std::collections::HashMap;
use std::future::{Future, IntoFuture};
use std::pin::Pin;

use nostr::{Filter, RelayUrl, SubscriptionId};

use super::filters_arg::FiltersArg;
use super::util::{convert_filters_arg_to_targets, convert_filters_arg_vec_to_map};
use crate::client::{Client, Error};
use crate::pool::Output;
use crate::relay::options::SubscribeAutoCloseOptions;

/// Subscribe to events
#[must_use = "Does nothing unless you await!"]
pub struct Subscribe<'client, 'url> {
    client: &'client Client,
    target: FiltersArg<'url>,
    id: Option<SubscriptionId>,
    auto_close: Option<SubscribeAutoCloseOptions>,
}

impl<'client, 'url> Subscribe<'client, 'url> {
    #[inline]
    pub(crate) fn new(client: &'client Client, target: FiltersArg<'url>) -> Self {
        Self {
            client,
            target,
            id: None,
            auto_close: None,
        }
    }

    /// Set a specific subscription ID
    #[inline]
    pub fn with_id(mut self, id: SubscriptionId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set auto-close conditions
    #[inline]
    pub fn close_on(mut self, opts: SubscribeAutoCloseOptions) -> Self {
        self.auto_close = Some(opts);
        self
    }

    async fn exec(self) -> Result<Output<SubscriptionId>, Error> {
        let targets: HashMap<RelayUrl, Vec<Filter>> = match &self.client.gossip {
            Some(gossip) => match self.target {
                // Gossip is configured and we need to break down filters before subscribing
                FiltersArg::Broadcast(filters) => {
                    self.client.break_down_filters(gossip, filters).await?
                }
                // The request is already targeted, skip gossip
                FiltersArg::Targeted(target) => convert_filters_arg_vec_to_map(target)?,
            },
            // No gossip configured: directly use the target
            None => convert_filters_arg_to_targets(&self.client.pool, self.target).await?,
        };

        Ok(self
            .client
            .pool
            .subscribe(targets, self.id, self.auto_close)
            .await?)
    }
}

impl<'client, 'url> IntoFuture for Subscribe<'client, 'url>
where
    'url: 'client,
{
    type Output = Result<Output<SubscriptionId>, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}
