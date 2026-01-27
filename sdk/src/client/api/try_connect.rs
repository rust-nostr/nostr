use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::time::Duration;

use super::output::Output;
use crate::blocking::Blocking;
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

    // TODO: return a Result? Replace the Output with something else?
    #[inline]
    async fn exec(self) -> Output<()> {
        self.client.pool.try_connect(self.timeout).await
    }
}

impl<'client> IntoFuture for TryConnect<'client> {
    type Output = Output<()>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

impl Blocking for TryConnect<'_> {}
