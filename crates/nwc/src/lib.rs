// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC client and zapper backend for Nostr apps

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::large_futures)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)] // TODO: remove when MSRV >= 1.72.0, required for `clippy::arc_with_non_send_sync`
#![allow(clippy::arc_with_non_send_sync)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub extern crate nostr;
pub extern crate nostr_zapper as zapper;

use async_trait::async_trait;
use async_utility::time;
use nostr::nips::nip47::{Request, Response};
use nostr_relay_pool::prelude::*;
use nostr_zapper::prelude::*;

pub mod error;
pub mod options;
pub mod prelude;

#[doc(hidden)]
pub use self::error::Error;
#[doc(hidden)]
pub use self::options::NostrWalletConnectOptions;

const ID: &str = "nwc";

/// Nostr Wallet Connect client
#[derive(Debug, Clone)]
pub struct NWC {
    uri: NostrWalletConnectURI,
    relay: Relay,
    opts: NostrWalletConnectOptions,
    bootstrapped: Arc<AtomicBool>,
}

impl NWC {
    /// New `NWC` client
    #[inline]
    pub fn new(uri: NostrWalletConnectURI) -> Self {
        Self::with_opts(uri, NostrWalletConnectOptions::default())
    }

    /// New `NWC` client with custom [`NostrWalletConnectOptions`].
    pub fn with_opts(uri: NostrWalletConnectURI, opts: NostrWalletConnectOptions) -> Self {
        Self {
            relay: Relay::with_opts(uri.relay_url.clone(), opts.relay.clone()),
            uri,
            opts,
            bootstrapped: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get relay status
    #[inline]
    pub fn status(&self) -> RelayStatus {
        self.relay.status()
    }

    /// Connect and subscribe
    async fn bootstrap(&self) -> Result<(), Error> {
        // Check if already bootstrapped
        if self.bootstrapped.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Connect
        self.relay.connect(None).await;

        let filter = Filter::new()
            .author(self.uri.public_key)
            .kind(Kind::WalletConnectResponse)
            .limit(0); // Limit to 0 means give me 0 events until EOSE

        // Subscribe
        self.relay
            .subscribe_with_id(
                SubscriptionId::new(ID),
                vec![filter],
                SubscribeOptions::default(),
            )
            .await?;

        // Mark as bootstrapped
        self.bootstrapped.store(true, Ordering::SeqCst);

        Ok(())
    }

    async fn send_request(&self, req: Request) -> Result<Response, Error> {
        // Bootstrap
        self.bootstrap().await?;

        // Convert request to event
        let event: Event = req.to_event(&self.uri)?;

        let mut notifications = self.relay.notifications();

        // Send request
        let id: EventId = self.relay.send_event(event).await?;

        time::timeout(Some(self.opts.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayNotification::Event { event, .. } = notification {
                    if event.kind == Kind::WalletConnectResponse
                        && event.tags.event_ids().next().copied() == Some(id)
                    {
                        return Ok(Response::from_event(&self.uri, &event)?);
                    }
                }
            }

            Err(Error::PrematureExit)
        })
        .await
        .ok_or(Error::Timeout)?
    }

    /// Pay invoice
    pub async fn pay_invoice<S>(&self, invoice: S) -> Result<String, Error>
    where
        S: Into<String>,
    {
        let req = Request::pay_invoice(PayInvoiceRequestParams {
            id: None,
            invoice: invoice.into(),
            amount: None,
        });
        let res: Response = self.send_request(req).await?;
        let PayInvoiceResponseResult { preimage } = res.to_pay_invoice()?;
        Ok(preimage)
    }

    /// Pay keysend
    pub async fn pay_keysend(
        &self,
        params: PayKeysendRequestParams,
    ) -> Result<PayKeysendResponseResult, Error> {
        let req = Request::pay_keysend(params);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_pay_keysend()?)
    }

    /// Create invoice
    pub async fn make_invoice(
        &self,
        params: MakeInvoiceRequestParams,
    ) -> Result<MakeInvoiceResponseResult, Error> {
        let req: Request = Request::make_invoice(params);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_make_invoice()?)
    }

    /// Lookup invoice
    pub async fn lookup_invoice(
        &self,
        params: LookupInvoiceRequestParams,
    ) -> Result<LookupInvoiceResponseResult, Error> {
        let req = Request::lookup_invoice(params);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_lookup_invoice()?)
    }

    /// List transactions
    pub async fn list_transactions(
        &self,
        params: ListTransactionsRequestParams,
    ) -> Result<Vec<LookupInvoiceResponseResult>, Error> {
        let req = Request::list_transactions(params);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_list_transactions()?)
    }

    /// Get balance
    pub async fn get_balance(&self) -> Result<u64, Error> {
        let req = Request::get_balance();
        let res: Response = self.send_request(req).await?;
        let GetBalanceResponseResult { balance } = res.to_get_balance()?;
        Ok(balance)
    }

    /// Get info
    pub async fn get_info(&self) -> Result<GetInfoResponseResult, Error> {
        let req = Request::get_info();
        let res: Response = self.send_request(req).await?;
        Ok(res.to_get_info()?)
    }

    /// Completely shutdown [NWC] client
    #[inline]
    pub fn shutdown(self) -> Result<(), Error> {
        Ok(self.relay.disconnect()?)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrZapper for NWC {
    #[inline]
    fn backend(&self) -> ZapperBackend {
        ZapperBackend::NWC
    }

    #[inline]
    async fn pay(&self, invoice: String) -> Result<(), ZapperError> {
        self.pay_invoice(invoice)
            .await
            .map_err(ZapperError::backend)?;
        Ok(())
    }
}
