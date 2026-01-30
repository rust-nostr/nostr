use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::time::Duration;

use super::output::Output;
use crate::client::Client;

/// Try to connect relays
#[must_use = "Does nothing unless you await!"]
pub struct TryConnect<'client> {
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
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.client.pool.try_connect(self.timeout).await })
    }
}

impl_blocking!(TryConnect<'_>);
