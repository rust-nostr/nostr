use std::collections::HashMap;
use std::future::{Future, IntoFuture};
use std::pin::Pin;

use nostr::{Filter, RelayUrl, SubscriptionId};

use super::output::Output;
use super::req_target::ReqTarget;
use super::util::build_targets;
use crate::client::{Client, Error};
use crate::relay::SubscribeAutoCloseOptions;

/// Subscribe to events
#[must_use = "Does nothing unless you await!"]
pub struct Subscribe<'client, 'url> {
    client: &'client Client,
    target: ReqTarget<'url>,
    id: Option<SubscriptionId>,
    auto_close: Option<SubscribeAutoCloseOptions>,
}

impl<'client, 'url> Subscribe<'client, 'url> {
    #[inline]
    pub(crate) fn new(client: &'client Client, target: ReqTarget<'url>) -> Self {
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
        // Build targets
        let targets: HashMap<RelayUrl, Vec<Filter>> =
            build_targets(self.client, self.target).await?;

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

impl_blocking!(for<'client, 'url> Subscribe<'client, 'url> where 'url: 'client);
