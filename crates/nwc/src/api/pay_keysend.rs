use std::future::IntoFuture;
use std::time::Duration;

use nostr::prelude::*;

use crate::NostrWalletConnect;
use crate::error::Error;

/// Pay keysend
#[must_use = "Does nothing unless you await!"]
pub struct PayKeysend<'client> {
    client: &'client NostrWalletConnect,
    request: PayKeysendRequest,
    timeout: Option<Duration>,
}

impl<'client> PayKeysend<'client> {
    pub(crate) fn new(client: &'client NostrWalletConnect, request: PayKeysendRequest) -> Self {
        Self {
            client,
            request,
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

impl<'client> IntoFuture for PayKeysend<'client> {
    type Output = Result<PayKeysendResponse, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let timeout: Duration = self.timeout.unwrap_or(self.client.timeout);
            let req = Request::pay_keysend(self.request);
            let res: Response = self.client.send_request(req, timeout).await?;
            Ok(res.to_pay_keysend()?)
        })
    }
}
