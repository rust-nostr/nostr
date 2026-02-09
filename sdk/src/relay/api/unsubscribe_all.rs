use std::future::IntoFuture;

use crate::future::BoxedFuture;
use crate::relay::{Error, Relay};

/// Unsubscribe from all REQs
#[must_use = "Does nothing unless you await!"]
pub struct UnsubscribeAll<'relay> {
    relay: &'relay Relay,
}

impl<'relay> UnsubscribeAll<'relay> {
    #[inline]
    pub(crate) fn new(relay: &'relay Relay) -> Self {
        Self { relay }
    }
}

impl<'relay> IntoFuture for UnsubscribeAll<'relay> {
    type Output = Result<(), Error>;
    type IntoFuture = BoxedFuture<'relay, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.relay.inner.unsubscribe_all().await })
    }
}

impl_blocking!(UnsubscribeAll<'_>);
