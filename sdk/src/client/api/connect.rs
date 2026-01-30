use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::time::Duration;

use crate::client::Client;

/// Connect relays
#[must_use = "Does nothing unless you await!"]
pub struct Connect<'client> {
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

    async fn exec(self) {
        self.client.pool.connect().await;

        if let Some(timeout) = self.wait {
            self.client.pool.wait_for_connection(timeout).await;
        }
    }
}

impl<'client> IntoFuture for Connect<'client> {
    type Output = ();
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl_blocking!(Connect<'_>);
