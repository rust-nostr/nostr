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
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub extern crate nostr;

use nostr::nips::nip47::{Notification, Request, Response};
use nostr_relay_pool::prelude::*;

pub mod error;
pub mod options;
pub mod prelude;

#[doc(hidden)]
pub use self::error::Error;
#[doc(hidden)]
pub use self::options::NostrWalletConnectOptions;

const NOTIFICATIONS_ID: &str = "nwc-notifications";

/// Nostr Wallet Connect client
#[derive(Debug, Clone)]
pub struct NWC {
    uri: NostrWalletConnectURI,
    pool: RelayPool,
    opts: NostrWalletConnectOptions,
    bootstrapped: Arc<AtomicBool>,
    notifications_subscribed: Arc<AtomicBool>,
}

impl NWC {
    /// New `NWC` client
    #[inline]
    pub fn new(uri: NostrWalletConnectURI) -> Self {
        Self::with_opts(uri, NostrWalletConnectOptions::default())
    }

    /// New `NWC` client with custom [`NostrWalletConnectOptions`].
    pub fn with_opts(uri: NostrWalletConnectURI, opts: NostrWalletConnectOptions) -> Self {
        let pool = match opts.monitor.as_ref() {
            Some(monitor) => RelayPool::builder().monitor(monitor.clone()).build(),
            None => RelayPool::default(),
        };

        Self {
            uri,
            pool,
            opts,
            bootstrapped: Arc::new(AtomicBool::new(false)),
            notifications_subscribed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get URI
    #[inline]
    pub fn uri(&self) -> &NostrWalletConnectURI {
        &self.uri
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

        // Mark as bootstrapped
        self.bootstrapped.store(true, Ordering::SeqCst);

        Ok(())
    }

    async fn send_request(&self, req: Request) -> Result<Response, Error> {
        // Bootstrap
        self.bootstrap().await?;

        tracing::debug!("Sending request '{}'", req.as_json());

        // Convert request to event
        let event: Event = req.to_event(&self.uri)?;

        // Construct the filter to wait for the response
        let filter = Filter::new()
            .author(self.uri.public_key)
            .kind(Kind::WalletConnectResponse)
            .event(event.id);

        // Subscribe to filter and create the stream
        let mut stream = self
            .pool
            .stream_events(filter, self.opts.timeout, ReqExitPolicy::WaitForEvents(1))
            .await?;

        // Send the request
        self.pool.send_event(&event).await?;

        // Wait for the response event
        let received_event: Event = stream.next().await.ok_or(Error::PrematureExit)?;

        // Parse response
        let response: Response = Response::from_event(&self.uri, &received_event)?;

        // Return response
        Ok(response)
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

    /// Subscribe to wallet notifications
    pub async fn subscribe_to_notifications(&self) -> Result<(), Error> {
        if self.notifications_subscribed.load(Ordering::SeqCst) {
            tracing::debug!("Already subscribed to notifications");
            return Ok(());
        }

        tracing::info!("Subscribing to wallet notifications...");

        self.bootstrap().await?;

        let client_keys = Keys::new(self.uri.secret.clone());
        let client_pubkey = client_keys.public_key();

        tracing::debug!("Client pubkey: {}", client_pubkey);
        tracing::debug!("Wallet service pubkey: {}", self.uri.public_key);

        let notification_filter = Filter::new()
            .author(self.uri.public_key)
            .pubkey(client_pubkey)
            .kind(Kind::WalletConnectNotification)
            .since(Timestamp::now());

        tracing::debug!("Notification filter: {:?}", notification_filter);

        self.pool
            .subscribe_with_id(
                SubscriptionId::new(NOTIFICATIONS_ID),
                notification_filter,
                SubscribeOptions::default(),
            )
            .await?;

        self.notifications_subscribed.store(true, Ordering::SeqCst);

        tracing::info!("Successfully subscribed to notifications");
        Ok(())
    }

    /// Handle incoming notifications with a callback function
    pub async fn handle_notifications<F, Fut>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(Notification) -> Fut,
        Fut: Future<Output = Result<bool>>,
    {
        let mut notifications = self.pool.notifications();

        while let Ok(notification) = notifications.recv().await {
            tracing::trace!("Received relay pool notification: {:?}", notification);

            match notification {
                RelayPoolNotification::Event {
                    subscription_id,
                    event,
                    ..
                } => {
                    tracing::debug!(
                        "Received event: kind={}, author={}, id={}",
                        event.kind,
                        event.pubkey,
                        event.id
                    );

                    if subscription_id.as_str() != NOTIFICATIONS_ID {
                        tracing::trace!("Ignoring event with subscription id: {}", subscription_id);
                        continue;
                    }

                    if event.kind != Kind::WalletConnectNotification {
                        tracing::trace!("Ignoring event with kind: {}", event.kind);
                        continue;
                    }

                    tracing::info!("Processing wallet notification event");

                    match Notification::from_event(&self.uri, &event) {
                        Ok(nip47_notification) => {
                            tracing::info!(
                                "Successfully parsed notification: {:?}",
                                nip47_notification.notification_type
                            );
                            let exit: bool = func(nip47_notification)
                                .await
                                .map_err(|e| Error::Handler(e.to_string()))?;
                            if exit {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse notification: {}", e);
                            tracing::debug!("Event content: {}", event.content);
                            return Err(Error::from(e));
                        }
                    }
                }
                RelayPoolNotification::Shutdown => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Unsubscribe from notifications
    pub async fn unsubscribe_from_notifications(&self) -> Result<(), Error> {
        self.pool
            .unsubscribe(&SubscriptionId::new(NOTIFICATIONS_ID))
            .await;
        self.notifications_subscribed.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Manually reconnect to a specific relay
    ///
    /// This function can be used to force a reconnection to a relay when automatic reconnection
    /// is disabled via [`RelayOptions::reconnect`].
    ///
    /// If the client is not bootstrapped, it will do nothing.
    pub async fn reconnect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        if !self.bootstrapped.load(Ordering::SeqCst) {
            return Ok(());
        }

        Ok(self.pool.connect_relay(url).await?)
    }

    /// Completely shutdown [NWC] client
    #[inline]
    pub async fn shutdown(self) {
        self.pool.disconnect().await
    }
}
