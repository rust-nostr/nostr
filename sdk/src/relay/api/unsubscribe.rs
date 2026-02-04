use std::future::IntoFuture;

use nostr::SubscriptionId;

use crate::future::BoxedFuture;
use crate::relay::{Error, Relay};

/// Unsubscribe from a REQ
#[must_use = "Does nothing unless you await!"]
pub struct Unsubscribe<'relay, 'id> {
    relay: &'relay Relay,
    id: &'id SubscriptionId,
}

impl<'relay, 'id> Unsubscribe<'relay, 'id> {
    #[inline]
    pub(crate) fn new(relay: &'relay Relay, id: &'id SubscriptionId) -> Self {
        Self { relay, id }
    }
}

impl<'relay, 'id> IntoFuture for Unsubscribe<'relay, 'id>
where
    'id: 'relay,
{
    type Output = Result<bool, Error>;
    type IntoFuture = BoxedFuture<'relay, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.relay.inner.unsubscribe(self.id).await })
    }
}
