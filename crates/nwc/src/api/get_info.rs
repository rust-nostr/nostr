use std::future::IntoFuture;
use std::time::Duration;

use nostr::prelude::*;

use crate::NostrWalletConnect;
use crate::error::Error;

/// Get info
#[must_use = "Does nothing unless you await!"]
pub struct GetInfo<'client> {
    client: &'client NostrWalletConnect,
    timeout: Option<Duration>,
}

impl<'client> GetInfo<'client> {
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

impl<'client> IntoFuture for GetInfo<'client> {
    type Output = Result<GetInfoResponse, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let timeout: Duration = self.timeout.unwrap_or(self.client.timeout);
            let req = Request::get_info();
            let res: Response = self.client.send_request(req, timeout).await?;
            Ok(res.to_get_info()?)
        })
    }
}
