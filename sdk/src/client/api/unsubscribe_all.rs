use std::future::IntoFuture;

use crate::client::{Client, Error, Output};
use crate::future::BoxedFuture;

/// Unsubscribe from all REQs
#[must_use = "Does nothing unless you await!"]
pub struct UnsubscribeAll<'client> {
    client: &'client Client,
}

impl<'client> UnsubscribeAll<'client> {
    #[inline]
    pub(crate) fn new(client: &'client Client) -> Self {
        Self { client }
    }
}

impl<'client> IntoFuture for UnsubscribeAll<'client> {
    type Output = Result<Output<()>, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Unsubscribe
            let output: Output<()> = self.client.pool.unsubscribe_all().await;

            Ok(output)
        })
    }
}
