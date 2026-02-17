use std::future::IntoFuture;
use std::time::Duration;

use super::output::Output;
use crate::client::Client;
use crate::future::BoxedFuture;

/// Try to connect relays
#[must_use = "Does nothing unless you await!"]
pub struct TryConnect<'client> {
    // --------------------------------------------------
    // WHEN ADDING NEW OPTIONS HERE,
    // REMEMBER TO UPDATE THE "Configuration" SECTION in
    // Client::try_connect DOC.
    // --------------------------------------------------
    client: &'client Client,
    timeout: Duration,
}

impl<'client> TryConnect<'client> {
    #[inline]
    pub(crate) fn new(client: &'client Client) -> Self {
        Self {
            client,
            timeout: Duration::from_secs(60),
        }
    }

    /// Timeout (default: 60 sec)
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl<'client> IntoFuture for TryConnect<'client> {
    // TODO: return a Result? Replace the Output with something else?
    type Output = Output<()>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.client.pool().try_connect(self.timeout).await })
    }
}
