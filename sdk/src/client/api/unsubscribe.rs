use std::future::{Future, IntoFuture};
use std::pin::Pin;

use nostr::SubscriptionId;

use super::blocking::Blocking;
use crate::client::{Client, Error, Output};

/// Unsubscribe from a REQ
#[must_use = "Does nothing unless you await!"]
pub struct Unsubscribe<'client, 'id> {
    client: &'client Client,
    id: &'id SubscriptionId,
}

impl<'client, 'id> Unsubscribe<'client, 'id> {
    #[inline]
    pub(crate) fn new(client: &'client Client, id: &'id SubscriptionId) -> Self {
        Self { client, id }
    }

    async fn exec(self) -> Result<Output<()>, Error> {
        // Unsubscribe
        let output: Output<()> = self.client.pool.unsubscribe(self.id).await;

        Ok(output)
    }
}

impl<'client, 'id> IntoFuture for Unsubscribe<'client, 'id>
where
    'id: 'client,
{
    type Output = Result<Output<()>, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl<'client, 'id> Blocking for Unsubscribe<'client, 'id> where 'id: 'client {}
