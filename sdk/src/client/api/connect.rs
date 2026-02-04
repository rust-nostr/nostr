use std::future::IntoFuture;
use std::time::Duration;

use crate::client::Client;
use crate::future::BoxedFuture;

/// Connect relays
#[must_use = "Does nothing unless you await!"]
pub struct Connect<'client> {
    // --------------------------------------------------
    // WHEN ADDING NEW OPTIONS HERE,
    // REMEMBER TO UPDATE THE "Configuration" SECTION in
    // Client::connect DOC.
    // --------------------------------------------------
    client: &'client Client,
    wait: Option<Duration>,
}

impl<'client> Connect<'client> {
    #[inline]
    pub(crate) fn new(client: &'client Client) -> Self {
        Self { client, wait: None }
    }

    /// Waits for relays connections
    ///
    /// Wait for relays connections at most for the specified `timeout`.
    /// The code continues when the relays are connected or the `timeout` is reached.
    #[inline]
    pub fn and_wait(mut self, timeout: Duration) -> Self {
        self.wait = Some(timeout);
        self
    }
}

impl<'client> IntoFuture for Connect<'client> {
    type Output = ();
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            self.client.pool.connect().await;

            if let Some(timeout) = self.wait {
                self.client.pool.wait_for_connection(timeout).await;
            }
        })
    }
}
