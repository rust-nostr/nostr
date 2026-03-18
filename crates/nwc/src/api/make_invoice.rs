use std::future::IntoFuture;
use std::time::Duration;

use nostr::prelude::*;

use crate::NostrWalletConnect;
use crate::error::Error;

/// Make invoice
#[must_use = "Does nothing unless you await!"]
pub struct MakeInvoice<'client> {
    client: &'client NostrWalletConnect,
    request: MakeInvoiceRequest,
    timeout: Option<Duration>,
}

impl<'client> MakeInvoice<'client> {
    pub(crate) fn new(client: &'client NostrWalletConnect, request: MakeInvoiceRequest) -> Self {
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

impl<'client> IntoFuture for MakeInvoice<'client> {
    type Output = Result<MakeInvoiceResponse, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let timeout: Duration = self.timeout.unwrap_or(self.client.timeout);
            let req = Request::make_invoice(self.request);
            let res: Response = self.client.send_request(req, timeout).await?;
            Ok(res.to_make_invoice()?)
        })
    }
}
