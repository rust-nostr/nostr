use std::future::IntoFuture;
use std::time::Duration;

use nostr::prelude::*;

use crate::NostrWalletConnect;
use crate::error::Error;

/// Lookup invoice
#[must_use = "Does nothing unless you await!"]
pub struct LookupInvoice<'client> {
    client: &'client NostrWalletConnect,
    request: LookupInvoiceRequest,
    timeout: Option<Duration>,
}

impl<'client> LookupInvoice<'client> {
    pub(crate) fn new(client: &'client NostrWalletConnect, request: LookupInvoiceRequest) -> Self {
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

impl<'client> IntoFuture for LookupInvoice<'client> {
    type Output = Result<LookupInvoiceResponse, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let timeout: Duration = self.timeout.unwrap_or(self.client.timeout);
            let req = Request::lookup_invoice(self.request);
            let res: Response = self.client.send_request(req, timeout).await?;
            Ok(res.to_lookup_invoice()?)
        })
    }
}
