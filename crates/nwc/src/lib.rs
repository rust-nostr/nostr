// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC client and zapper backend for Nostr apps

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)]
#![allow(clippy::arc_with_non_send_sync)]

use std::time::Duration;

pub extern crate nostr;
pub extern crate nostr_zapper as zapper;

use async_utility::time;
use nostr::nips::nip47::{
    GetBalanceResponseResult, GetInfoResponseResult, ListTransactionsRequestParams,
    LookupInvoiceRequestParams, LookupInvoiceResponseResult, MakeInvoiceRequestParams,
    MakeInvoiceResponseResult, NostrWalletConnectURI, PayInvoiceRequestParams,
    PayInvoiceResponseResult, PayKeysendRequestParams, PayKeysendResponseResult, Request, Response,
};
use nostr::{Event, EventId, Filter, Kind, Timestamp};
use nostr_relay_pool::{Relay, RelayNotification, RelaySendOptions, SubscribeOptions};
use nostr_zapper::{async_trait, NostrZapper, ZapperBackend};

pub mod error;
pub mod options;
pub mod prelude;

pub use self::error::Error;
pub use self::options::NostrWalletConnectOptions;

/// Nostr Wallet Connect client
#[derive(Debug, Clone)]
pub struct NWC {
    uri: NostrWalletConnectURI,
    relay: Relay,
    opts: NostrWalletConnectOptions,
}

impl NWC {
    /// Compose new [NWC] client
    pub async fn new(uri: NostrWalletConnectURI) -> Result<Self, Error> {
        Self::with_opts(uri, NostrWalletConnectOptions::default()).await
    }

    /// Compose new [NWC] client with [NostrWalletConnectOptions]
    pub async fn with_opts(
        uri: NostrWalletConnectURI,
        opts: NostrWalletConnectOptions,
    ) -> Result<Self, Error> {
        // Compose relay
        let relay = Relay::with_opts(uri.relay_url.clone(), opts.relay.clone());
        relay.connect(Some(Duration::from_secs(10))).await;

        let this = Self { uri, relay, opts };

        // Subscribe
        this.subscribe().await?;

        Ok(this)
    }

    async fn subscribe(&self) -> Result<(), Error> {
        let filter = Filter::new()
            .author(self.uri.public_key)
            .kind(Kind::WalletConnectResponse)
            .since(Timestamp::now());

        // Subscribe
        self.relay
            .subscribe(vec![filter], SubscribeOptions::default())
            .await?;

        Ok(())
    }

    async fn send_request(&self, req: Request) -> Result<Response, Error> {
        // Convert request to event
        let event: Event = req.to_event(&self.uri)?;
        let event_id: EventId = event.id;

        let mut notifications = self.relay.notifications();

        // Send request
        self.relay
            .send_event(event, RelaySendOptions::new())
            .await?;

        time::timeout(Some(self.opts.timeout), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::WalletConnectResponse
                        && event.event_ids().next().copied() == Some(event_id)
                    {
                        return Ok(Response::from_event(&self.uri, &event)?);
                    }
                }
            }

            Err(Error::Timeout)
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
    pub async fn shutdown(self) -> Result<(), Error> {
        Ok(self.relay.terminate().await?)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrZapper for NWC {
    type Err = Error;

    fn backend(&self) -> ZapperBackend {
        ZapperBackend::NWC
    }

    async fn pay(&self, invoice: String) -> Result<(), Self::Err> {
        self.pay_invoice(invoice).await?;
        Ok(())
    }
}
