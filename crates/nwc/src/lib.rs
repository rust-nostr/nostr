// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC client and zapper backend for Nostr apps

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::large_futures)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)] // TODO: remove when MSRV >= 1.72.0, required for `clippy::arc_with_non_send_sync`
#![allow(clippy::arc_with_non_send_sync)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub extern crate nostr;

use async_utility::time;
use nostr::nips::nip47::{Request, Response};
use nostr_relay_pool::prelude::*;

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
    pool: RelayPool,
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
            uri,
            pool: RelayPool::default(),
            opts,
            bootstrapped: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get relays status
    pub async fn status(&self) -> HashMap<RelayUrl, RelayStatus> {
        let relays = self.pool.relays().await;
        relays.into_iter().map(|(u, r)| (u, r.status())).collect()
    }

    /// Connect and subscribe
    async fn bootstrap(&self) -> Result<(), Error> {
        // Check if already bootstrapped
        if self.bootstrapped.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Add relays
        for url in self.uri.relays.iter() {
            self.pool.add_relay(url, self.opts.relay.clone()).await?;
        }

        // Connect to relays
        self.pool.connect().await;

        let filter = Filter::new()
            .author(self.uri.public_key)
            .kind(Kind::WalletConnectResponse)
            .limit(0); // Limit to 0 means give me 0 events until EOSE

        // Subscribe
        self.pool
            .subscribe_with_id(SubscriptionId::new(ID), filter, SubscribeOptions::default())
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

        let mut notifications = self.pool.notifications();

        // Send request
        let output: Output<EventId> = self.pool.send_event(&event).await?;

        time::timeout(Some(self.opts.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::WalletConnectResponse
                        && event.tags.event_ids().next() == Some(output.id())
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
    pub async fn pay_invoice(
        &self,
        request: PayInvoiceRequest,
    ) -> Result<PayInvoiceResponse, Error> {
        let req = Request::pay_invoice(request);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_pay_invoice()?)
    }

    /// Pay keysend
    pub async fn pay_keysend(
        &self,
        request: PayKeysendRequest,
    ) -> Result<PayKeysendResponse, Error> {
        let req = Request::pay_keysend(request);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_pay_keysend()?)
    }

    /// Create invoice
    pub async fn make_invoice(
        &self,
        request: MakeInvoiceRequest,
    ) -> Result<MakeInvoiceResponse, Error> {
        let req: Request = Request::make_invoice(request);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_make_invoice()?)
    }

    /// Lookup invoice
    pub async fn lookup_invoice(
        &self,
        request: LookupInvoiceRequest,
    ) -> Result<LookupInvoiceResponse, Error> {
        let req = Request::lookup_invoice(request);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_lookup_invoice()?)
    }

    /// List transactions
    pub async fn list_transactions(
        &self,
        params: ListTransactionsRequest,
    ) -> Result<Vec<LookupInvoiceResponse>, Error> {
        let req = Request::list_transactions(params);
        let res: Response = self.send_request(req).await?;
        Ok(res.to_list_transactions()?)
    }

    /// Get balance (msat)
    pub async fn get_balance(&self) -> Result<u64, Error> {
        let req = Request::get_balance();
        let res: Response = self.send_request(req).await?;
        let GetBalanceResponse { balance } = res.to_get_balance()?;
        Ok(balance)
    }

    /// Get info
    pub async fn get_info(&self) -> Result<GetInfoResponse, Error> {
        let req = Request::get_info();
        let res: Response = self.send_request(req).await?;
        Ok(res.to_get_info()?)
    }

    /// Completely shutdown [NWC] client
    #[inline]
    pub async fn shutdown(self) {
        self.pool.disconnect().await
    }
}
