use std::future::IntoFuture;

use nostr::SubscriptionId;

use crate::client::{Client, Error, Output};
use crate::future::BoxedFuture;

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
}

impl<'client, 'id> IntoFuture for Unsubscribe<'client, 'id>
where
    'id: 'client,
{
    type Output = Result<Output<()>, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Unsubscribe
            let output: Output<()> = self.client.pool().unsubscribe(self.id).await;

            Ok(output)
        })
    }
}
