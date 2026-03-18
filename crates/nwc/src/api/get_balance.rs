use std::future::IntoFuture;
use std::time::Duration;

use nostr::prelude::*;

use crate::NostrWalletConnect;
use crate::error::Error;

/// Get balance
#[must_use = "Does nothing unless you await!"]
pub struct GetBalance<'client> {
    client: &'client NostrWalletConnect,
    timeout: Option<Duration>,
}

impl<'client> GetBalance<'client> {
    pub(crate) fn new(client: &'client NostrWalletConnect) -> Self {
        Self {
            client,
            timeout: None,
        }
    }

    /// Set timeout
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl<'client> IntoFuture for GetBalance<'client> {
    type Output = Result<GetBalanceResponse, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let timeout: Duration = self.timeout.unwrap_or(self.client.timeout);
            let req = Request::get_balance();
            let res: Response = self.client.send_request(req, timeout).await?;
            Ok(res.to_get_balance()?)
        })
    }
}
