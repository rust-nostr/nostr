use std::future::IntoFuture;
use std::time::Duration;

use nostr::prelude::*;

use crate::NostrWalletConnect;
use crate::error::Error;

/// Pay invoice
#[must_use = "Does nothing unless you await!"]
pub struct PayInvoice<'client> {
    client: &'client NostrWalletConnect,
    request: PayInvoiceRequest,
    timeout: Option<Duration>,
}

impl<'client> PayInvoice<'client> {
    pub(crate) fn new(client: &'client NostrWalletConnect, request: PayInvoiceRequest) -> Self {
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

impl<'client> IntoFuture for PayInvoice<'client> {
    type Output = Result<PayInvoiceResponse, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let timeout: Duration = self.timeout.unwrap_or(self.client.timeout);
            let req = Request::pay_invoice(self.request);
            let res: Response = self.client.send_request(req, timeout).await?;
            Ok(res.to_pay_invoice()?)
        })
    }
}
