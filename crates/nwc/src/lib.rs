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
    GetBalanceResponseResult, GetInfoResponseResult, MakeInvoiceRequestParams,
    MakeInvoiceResponseResult, Method, NostrWalletConnectURI, PayInvoiceRequestParams,
    PayInvoiceResponseResult, Request, RequestParams, Response,
};
use nostr::{Filter, Kind, SubscriptionId};
use nostr_relay_pool::{FilterOptions, RelayPool, RelayPoolNotification, RelaySendOptions};
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
    pool: RelayPool,
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
        // Compose pool
        let pool = RelayPool::new(opts.pool);
        pool.add_relay(&uri.relay_url, opts.relay).await?;
        pool.connect(Some(Duration::from_secs(10))).await;

        Ok(Self { uri, pool })
    }

    /// Create invoice
    pub async fn make_invoice(
        &self,
        satoshi: u64,
        description: Option<String>,
        expiry: Option<u64>,
    ) -> Result<String, Error> {
        // Compose NWC request event
        let req = Request {
            method: Method::MakeInvoice,
            params: RequestParams::MakeInvoice(MakeInvoiceRequestParams {
                amount: satoshi * 1000,
                description,
                description_hash: None,
                expiry,
            }),
        };
        let event = req.to_event(&self.uri)?;
        let event_id = event.id;

        // Subscribe
        let relay = self.pool.relay(&self.uri.relay_url).await?;
        let id = SubscriptionId::generate();
        let filter = Filter::new()
            .author(self.uri.public_key)
            .kind(Kind::WalletConnectResponse)
            .event(event_id)
            .limit(1);

        // Subscribe
        relay
            .send_req(
                id,
                vec![filter],
                Some(FilterOptions::WaitForEventsAfterEOSE(1)),
            )
            .await?;

        let mut notifications = self.pool.notifications();

        // Send request
        self.pool
            .send_event_to([&self.uri.relay_url], event, RelaySendOptions::new())
            .await?;

        time::timeout(Some(Duration::from_secs(10)), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::WalletConnectResponse
                        && event.event_ids().next().copied() == Some(event_id)
                    {
                        let res = Response::from_event(&self.uri, &event)?;
                        let MakeInvoiceResponseResult { invoice, .. } = res.to_make_invoice()?;
                        return Ok(invoice);
                    }
                }
            }

            Err(Error::Timeout)
        })
        .await
        .ok_or(Error::Timeout)?
    }

    /// Pay invoice
    pub async fn send_payment(&self, invoice: String) -> Result<(), Error> {
        // Compose NWC request event
        let req = Request {
            method: Method::PayInvoice,
            params: RequestParams::PayInvoice(PayInvoiceRequestParams {
                id: None,
                invoice,
                amount: None,
            }),
        };
        let event = req.to_event(&self.uri)?;
        let event_id = event.id;

        // Subscribe
        let relay = self.pool.relay(&self.uri.relay_url).await?;
        let id = SubscriptionId::generate();
        let filter = Filter::new()
            .author(self.uri.public_key)
            .kind(Kind::WalletConnectResponse)
            .event(event_id)
            .limit(1);

        // Subscribe
        relay
            .send_req(
                id,
                vec![filter],
                Some(FilterOptions::WaitForEventsAfterEOSE(1)),
            )
            .await?;

        let mut notifications = self.pool.notifications();

        // Send request
        self.pool
            .send_event_to([&self.uri.relay_url], event, RelaySendOptions::new())
            .await?;

        time::timeout(Some(Duration::from_secs(10)), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::WalletConnectResponse
                        && event.event_ids().next().copied() == Some(event_id)
                    {
                        let res = Response::from_event(&self.uri, &event)?;
                        let PayInvoiceResponseResult { preimage } = res.to_pay_invoice()?;
                        tracing::info!("Invoice paid! Preimage: {preimage}");
                        break;
                    }
                }
            }

            Ok::<(), Error>(())
        })
        .await
        .ok_or(Error::Timeout)?
    }

    /// Get balance
    pub async fn get_balance(&self) -> Result<u64, Error> {
        // Compose NWC request event
        let req = Request::get_balance();
        let event = req.to_event(&self.uri)?;
        let event_id = event.id;

        // Subscribe
        let relay = self.pool.relay(&self.uri.relay_url).await?;
        let id = SubscriptionId::generate();
        let filter = Filter::new()
            .author(self.uri.public_key)
            .kind(Kind::WalletConnectResponse)
            .event(event_id)
            .limit(1);

        // Subscribe
        relay
            .send_req(
                id,
                vec![filter],
                Some(FilterOptions::WaitForEventsAfterEOSE(1)),
            )
            .await?;

        let mut notifications = self.pool.notifications();

        // Send request
        self.pool
            .send_event_to([&self.uri.relay_url], event, RelaySendOptions::new())
            .await?;

        time::timeout(Some(Duration::from_secs(10)), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::WalletConnectResponse
                        && event.event_ids().next().copied() == Some(event_id)
                    {
                        let res = Response::from_event(&self.uri, &event)?;
                        let GetBalanceResponseResult { balance } = res.to_get_balance()?;
                        return Ok(balance);
                    }
                }
            }

            Err(Error::Timeout)
        })
        .await
        .ok_or(Error::Timeout)?
    }

    /// Get info
    pub async fn get_info(&self) -> Result<GetInfoResponseResult, Error> {
        // Compose NWC request event
        let req = Request::get_info();
        let event = req.to_event(&self.uri)?;
        let event_id = event.id;

        // Subscribe
        let relay = self.pool.relay(&self.uri.relay_url).await?;
        let id = SubscriptionId::generate();
        let filter = Filter::new()
            .author(self.uri.public_key)
            .kind(Kind::WalletConnectResponse)
            .event(event_id)
            .limit(1);

        // Subscribe
        relay
            .send_req(
                id,
                vec![filter],
                Some(FilterOptions::WaitForEventsAfterEOSE(1)),
            )
            .await?;

        let mut notifications = self.pool.notifications();

        // Send request
        self.pool
            .send_event_to([&self.uri.relay_url], event, RelaySendOptions::new())
            .await?;

        time::timeout(Some(Duration::from_secs(10)), async {
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind() == Kind::WalletConnectResponse
                        && event.event_ids().next().copied() == Some(event_id)
                    {
                        let res = Response::from_event(&self.uri, &event)?;
                        return Ok(res.to_get_info()?);
                    }
                }
            }

            Err(Error::Timeout)
        })
        .await
        .ok_or(Error::Timeout)?
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NostrZapper for NWC {
    type Err = Error;

    fn backend(&self) -> ZapperBackend {
        ZapperBackend::NWC
    }

    async fn pay_invoice(&self, invoice: String) -> Result<(), Self::Err> {
        self.send_payment(invoice).await
    }
}
