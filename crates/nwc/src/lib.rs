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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub extern crate nostr;

use nostr::nips::nip47::{Notification, Request, Response};
use nostr_sdk::prelude::*;

pub mod builder;
pub mod error;
pub mod prelude;

use self::builder::NostrWalletConnectBuilder;
#[doc(hidden)]
pub use self::error::Error;

const NOTIFICATIONS_ID: &str = "nwc-notifications";

#[allow(missing_docs)]
#[deprecated(since = "0.45.0", note = "Use NostrWalletConnect instead")]
pub type NWC = NostrWalletConnect;

/// Nostr Wallet Connect client
#[derive(Debug, Clone)]
pub struct NostrWalletConnect {
    uri: NostrWalletConnectUri,
    client: Client,
    timeout: Duration,
    relay_opts: RelayOptions,
    bootstrapped: Arc<AtomicBool>,
    notifications_subscribed: Arc<AtomicBool>,
}

impl NostrWalletConnect {
    /// Construct a new client.
    ///
    /// Use [`NostrWalletConnect::builder`] for customizing the client.
    #[inline]
    pub fn new(uri: NostrWalletConnectUri) -> Self {
        Self::builder(uri).build()
    }

    /// Construct a new Nostr Wallet Connect client builder.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::time::Duration;
    /// use nwc::prelude::*;
    ///
    /// # let uri = NostrWalletConnectUri::parse("nostr+walletconnect://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?secret=71a8c14c1407c113601079c4302dab36460f0ccd0ad506f1f2dc73b5100e4f3c&relay=wss%3A%2F%2Frelay.damus.io").unwrap();
    /// let nwc = NostrWalletConnect::builder(uri).timeout(Duration::from_secs(30)).build();
    /// # let _ = nwc;
    /// ```
    #[inline]
    pub fn builder(uri: NostrWalletConnectUri) -> NostrWalletConnectBuilder {
        NostrWalletConnectBuilder::new(uri)
    }

    fn from_builder(builder: NostrWalletConnectBuilder) -> Self {
        let client: Client = match builder.monitor {
            Some(monitor) => Client::builder().monitor(monitor).build(),
            None => Client::default(),
        };

        Self {
            uri: builder.uri,
            client,
            timeout: builder.timeout,
            relay_opts: builder.relay,
            bootstrapped: Arc::new(AtomicBool::new(false)),
            notifications_subscribed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get URI
    #[inline]
    pub fn uri(&self) -> &NostrWalletConnectUri {
        &self.uri
    }

    /// Get relays status
    pub async fn status(&self) -> HashMap<RelayUrl, RelayStatus> {
        let relays = self.client.relays().await;
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
            self.client
                .add_relay(url)
                .opts(self.relay_opts.clone())
                .await?;
        }

        // Connect to relays
        self.client.connect().await;

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
            .client
            .stream_events(filter)
            .timeout(self.timeout)
            .policy(ReqExitPolicy::WaitForEvents(1))
            .await?;

        // Send the request
        self.client.send_event(&event).await?;

        // Wait for the response
        let (_, res) = stream.next().await.ok_or(Error::PrematureExit)?;

        // Unwrap event
        let received_event: Event = res?;

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

        self.client
            .subscribe(notification_filter)
            .with_id(SubscriptionId::new(NOTIFICATIONS_ID))
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
        let mut notifications = self.client.notifications();

        while let Some(notification) = notifications.next().await {
            tracing::trace!("Received a client notification: {:?}", notification);

            match notification {
                ClientNotification::Event {
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
                ClientNotification::Shutdown => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Unsubscribe from notifications
    pub async fn unsubscribe_from_notifications(&self) -> Result<(), Error> {
        self.client
            .unsubscribe(&SubscriptionId::new(NOTIFICATIONS_ID))
            .await?;
        self.notifications_subscribed.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Manually reconnect to a specific relay
    ///
    /// This function can be used to force a reconnection to a relay when the automatic reconnection
    /// is disabled via [`RelayOptions::reconnect`].
    ///
    /// If the client is not bootstrapped, it will do nothing.
    pub async fn reconnect_relay<'a, U>(&self, url: U) -> Result<(), Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        if !self.bootstrapped.load(Ordering::SeqCst) {
            return Ok(());
        }

        Ok(self.client.connect_relay(url).await?)
    }

    /// Completely shutdown
    #[inline]
    pub async fn shutdown(self) {
        self.client.disconnect().await
    }
}
