// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Client

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_wsocket::ConnectionMode;
use futures::StreamExt;
use nostr::prelude::*;
use nostr_database::prelude::*;
use nostr_gossip::{GossipListKind, GossipPublicKeyStatus, NostrGossip};
use tokio::sync::{broadcast, Semaphore};

mod api;
mod builder;
mod error;
mod gossip;
mod middleware;
mod notification;
mod options;

pub use self::api::*;
pub use self::builder::*;
pub use self::error::Error;
use self::gossip::{BrokenDownFilters, GossipFilterPattern, GossipWrapper};
use self::middleware::AdmissionPolicyMiddleware;
pub use self::notification::*;
pub use self::options::*;
use crate::monitor::Monitor;
use crate::pool::{RelayPool, RelayPoolBuilder};
use crate::relay::options::{RelayOptions, SyncDirection, SyncOptions};
use crate::relay::{Reconciliation, Relay, RelayCapabilities, ReqExitPolicy};

/// Nostr client
#[derive(Debug, Clone)]
pub struct Client {
    pool: Arc<RelayPool>,
    gossip: Option<GossipWrapper>,
    opts: ClientOptions,
    /// Semaphore used to limit the number of gossip checks and syncs
    gossip_sync: Arc<Semaphore>,
}

impl Default for Client {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// Construct a new default client
    ///
    /// Use the [`Client::builder`] to configure the client (i.e., set a signer).
    #[inline]
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Construct client
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::time::Duration;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// let signer = Keys::generate();
    /// let client: Client = Client::builder().signer(signer).build();
    /// ```
    #[inline]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    fn from_builder(builder: ClientBuilder) -> Self {
        // Construct admission policy middleware
        let admit_policy_wrapper = AdmissionPolicyMiddleware {
            gossip: builder.gossip.clone(),
            external_policy: builder.admit_policy,
        };

        // Construct relay pool builder
        let pool_builder: RelayPoolBuilder = RelayPoolBuilder {
            websocket_transport: builder.websocket_transport,
            admit_policy: Some(Arc::new(admit_policy_wrapper)),
            monitor: builder.monitor,
            database: builder.database,
            signer: builder.signer,
            max_relays: builder.opts.max_relays,
            nip42_auto_authentication: builder.opts.nip42_auto_authentication,
            notification_channel_size: builder.opts.notification_channel_size,
        };

        // Construct client
        Self {
            pool: Arc::new(pool_builder.build()),
            gossip: builder.gossip.map(GossipWrapper::new),
            opts: builder.opts,
            // Allow only one gossip check and sync at a time
            gossip_sync: Arc::new(Semaphore::new(1)),
        }
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn automatic_authentication(&self, enable: bool) {
        self.pool.state().automatic_authentication(enable);
    }

    /// Check if signer is configured
    #[inline]
    pub async fn has_signer(&self) -> bool {
        self.pool.state().has_signer().await
    }

    /// Get current nostr signer
    ///
    /// # Errors
    ///
    /// Returns an error if the signer isn't set.
    #[inline]
    pub async fn signer(&self) -> Result<Arc<dyn NostrSigner>, Error> {
        Ok(self.pool.state().signer().await?)
    }

    /// Set nostr signer
    #[inline]
    pub async fn set_signer<T>(&self, signer: T)
    where
        T: IntoNostrSigner,
    {
        self.pool.state().set_signer(signer).await;
    }

    /// Unset nostr signer
    #[inline]
    pub async fn unset_signer(&self) {
        self.pool.state().unset_signer().await;
    }

    /// Retrieves the client's public key
    ///
    /// # Errors
    ///
    /// - If the signer isn't set.
    /// - Error by the signer.
    #[inline]
    pub async fn public_key(&self) -> Result<PublicKey, Error> {
        Ok(self.signer().await?.get_public_key().await?)
    }

    /// Get database
    #[inline]
    pub fn database(&self) -> &Arc<dyn NostrDatabase> {
        self.pool.database()
    }

    /// Get the relay monitor
    #[inline]
    pub fn monitor(&self) -> Option<&Monitor> {
        self.pool.monitor()
    }

    /// Reset the client
    ///
    /// This method resets the client to simplify the switch to another account.
    ///
    /// This method will:
    /// * unsubscribe from all subscriptions
    /// * disconnect and force remove all relays
    /// * unset the signer
    ///
    /// This method will NOT:
    /// * reset [`ClientOptions`]
    /// * remove the database
    /// * clear the gossip graph
    pub async fn reset(&self) {
        let _ = self.unsubscribe_all().await;
        let _ = self.remove_all_relays().force().await;
        self.unset_signer().await;
    }

    /// Check if the client is shutting down
    #[inline]
    pub fn is_shutdown(&self) -> bool {
        self.pool.is_shutdown()
    }

    /// Explicitly shutdown the client
    ///
    /// This method will shut down the client and all its relays.
    #[inline]
    pub async fn shutdown(&self) {
        self.pool.shutdown().await
    }

    /// Get new notification listener
    ///
    /// <div class="warning">When you call this method, you subscribe to the notifications channel from that precise moment. Anything received by relay/s before that moment is not included in the channel!</div>
    #[inline]
    pub fn notifications(&self) -> broadcast::Receiver<ClientNotification> {
        self.pool.notifications()
    }

    /// Get relays with [`RelayCapabilities::READ`] or [`RelayCapabilities::WRITE`] capabilities
    #[inline]
    pub fn relays(&self) -> GetRelays {
        GetRelays::new(self)
    }

    /// Get a previously added [`Relay`] by URL.
    ///
    /// It returns the relay **only if it has already been added**
    /// to the client via [`Client::add_relay`].
    ///
    /// - Returns `Ok(None)` if the relay is not found in the pool.
    /// - Returns `Err(_)` if the provided URL cannot be parsed as a relay URL.
    ///
    /// # Example
    ///
    /// ```
    /// # use nostr_sdk::prelude::*;
    /// # async fn example() -> Result<()> {
    /// # let client = Client::default();
    /// // Not added yet
    /// let relay = client.relay("wss://relay.example.com").await?;
    /// assert!(relay.is_none());
    ///
    /// // Add it
    /// client.add_relay("wss://relay.example.com").await?;
    ///
    /// // Now it can be retrieved
    /// let relay = client.relay("wss://relay.example.com").await?;
    /// assert!(relay.is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn relay<'a, U>(&self, url: U) -> Result<Option<Relay>, Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        let url: RelayUrlArg<'a> = url.into();
        let url: Cow<RelayUrl> = url.try_as_relay_url()?;
        Ok(self.pool.relay(&url).await)
    }

    fn compose_relay_opts<'a>(&self, _url: &'a RelayUrlArg<'a>) -> RelayOptions {
        let mut opts: RelayOptions = RelayOptions::new();

        // Set connection mode
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(url) = _url.try_as_relay_url() {
            match &self.opts.connection.mode {
                ConnectionMode::Direct => {}
                ConnectionMode::Proxy(..) => match self.opts.connection.target {
                    ConnectionTarget::All => {
                        opts = opts.connection_mode(self.opts.connection.mode.clone());
                    }
                    ConnectionTarget::Onion => {
                        if url.is_onion() {
                            opts = opts.connection_mode(self.opts.connection.mode.clone())
                        }
                    }
                },
                #[cfg(feature = "tor")]
                ConnectionMode::Tor { .. } => match self.opts.connection.target {
                    ConnectionTarget::All => {
                        opts = opts.connection_mode(self.opts.connection.mode.clone());
                    }
                    ConnectionTarget::Onion => {
                        if url.is_onion() {
                            opts = opts.connection_mode(self.opts.connection.mode.clone())
                        }
                    }
                },
            };
        }

        // Set sleep when idle
        match self.opts.sleep_when_idle {
            // Do nothing
            SleepWhenIdle::Disabled => {}
            // Enable: update relay options
            SleepWhenIdle::Enabled { timeout } => {
                opts = opts.sleep_when_idle(true).idle_timeout(timeout);
            }
        };

        // Set limits
        opts.limits(self.opts.relay_limits.clone())
            .max_avg_latency(self.opts.max_avg_latency)
            .verify_subscriptions(self.opts.verify_subscriptions)
            .ban_relay_on_mismatch(self.opts.ban_relay_on_mismatch)
    }

    /// Add relay
    ///
    /// By default, relays added with this method will have both [`RelayCapabilities::READ`] and [`RelayCapabilities::WRITE`] capabilities enabled.
    /// If the relay already exists, the capabilities will be updated and `false` returned.
    ///
    /// To add a relay with specific capabilities, use [`AddRelay::capabilities`].
    ///
    /// Connection is **NOT** automatically started with relay!
    #[inline]
    pub fn add_relay<'client, 'url, U>(&'client self, url: U) -> AddRelay<'client, 'url>
    where
        U: Into<RelayUrlArg<'url>>,
    {
        let url: RelayUrlArg<'url> = url.into();
        let opts: RelayOptions = self.compose_relay_opts(&url);
        AddRelay::new(self, url).opts(opts)
    }

    /// Add discovery relay
    ///
    /// If relay already exists, this method automatically add the [`RelayCapabilities::DISCOVERY`] flag to it and return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[deprecated(
        since = "0.45.0",
        note = "Use `Client::add_relay(url).capabilities(RelayCapabilities::DISCOVERY)` instead."
    )]
    pub async fn add_discovery_relay<'u, U>(&self, url: U) -> Result<bool, Error>
    where
        U: Into<RelayUrlArg<'u>>,
    {
        self.add_relay(url)
            .capabilities(RelayCapabilities::DISCOVERY)
            .await
    }

    /// Add read relay
    #[deprecated(
        since = "0.45.0",
        note = "Use `Client::add_relay(url).capabilities(RelayCapabilities::READ)` instead."
    )]
    pub async fn add_read_relay<'u, U>(&self, url: U) -> Result<bool, Error>
    where
        U: Into<RelayUrlArg<'u>>,
    {
        self.add_relay(url)
            .capabilities(RelayCapabilities::READ)
            .await
    }

    /// Add write relay
    #[deprecated(
        since = "0.45.0",
        note = "Use `Client::add_relay(url).capabilities(RelayCapabilities::WRITE)` instead."
    )]
    pub async fn add_write_relay<'u, U>(&self, url: U) -> Result<bool, Error>
    where
        U: Into<RelayUrlArg<'u>>,
    {
        self.add_relay(url)
            .capabilities(RelayCapabilities::WRITE)
            .await
    }

    /// Add gossip relay
    #[deprecated(
        since = "0.45.0",
        note = "Use `Client::add_relay(url).capabilities(RelayCapabilities::GOSSIP)` instead."
    )]
    pub async fn add_gossip_relay<'u, U>(&self, url: U) -> Result<bool, Error>
    where
        U: Into<RelayUrlArg<'u>>,
    {
        self.add_relay(url)
            .capabilities(RelayCapabilities::GOSSIP)
            .await
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has [`RelayCapabilities::GOSSIP`], it will not be removed from the pool and its
    /// capabilities will be updated (remove [`RelayCapabilities::READ`],
    /// [`RelayCapabilities::WRITE`] and [`RelayCapabilities::DISCOVERY`] capabilities).
    #[inline]
    pub fn remove_relay<'p, 'u, U>(&'p self, url: U) -> RemoveRelay<'p, 'u>
    where
        U: Into<RelayUrlArg<'u>>,
    {
        RemoveRelay::new(self, url.into())
    }

    /// Force remove and disconnect relay
    ///
    /// Note: this method will remove the relay, also if it's in use for the gossip model or other service!
    #[deprecated(since = "0.45.0", note = "use `remove_relay(url).force()` instead")]
    pub async fn force_remove_relay<'u, U>(&self, url: U) -> Result<(), Error>
    where
        U: Into<RelayUrlArg<'u>>,
    {
        self.remove_relay(url).force().await
    }

    /// Disconnect and remove all relays
    ///
    /// Some relays (i.e., the gossip ones) will not be disconnected and removed unless you
    /// use [`RemoveAllRelays::force()`].
    #[inline]
    pub fn remove_all_relays(&self) -> RemoveAllRelays<'_> {
        RemoveAllRelays::new(self)
    }

    /// Disconnect and force remove all relays
    #[deprecated(
        since = "0.45.0",
        note = "use `remove_all_relays(url).force()` instead"
    )]
    pub async fn force_remove_all_relays(&self) {
        let _ = self.remove_all_relays().force().await;
    }

    /// Connect to a previously added relay
    #[inline]
    pub async fn connect_relay<'a, U>(&self, url: U) -> Result<(), Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        let url: RelayUrlArg<'a> = url.into();
        let url: Cow<RelayUrl> = url.try_as_relay_url()?;
        Ok(self.pool.connect_relay(&url).await?)
    }

    /// Try to connect to a previously added relay
    #[inline]
    pub async fn try_connect_relay<'a, U>(&self, url: U, timeout: Duration) -> Result<(), Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        let url: RelayUrlArg<'a> = url.into();
        let url: Cow<RelayUrl> = url.try_as_relay_url()?;
        Ok(self.pool.try_connect_relay(&url, timeout).await?)
    }

    /// Disconnect relay
    #[inline]
    pub async fn disconnect_relay<'a, U>(&self, url: U) -> Result<(), Error>
    where
        U: Into<RelayUrlArg<'a>>,
    {
        let url: RelayUrlArg<'a> = url.into();
        let url: Cow<RelayUrl> = url.try_as_relay_url()?;
        Ok(self.pool.disconnect_relay(&url).await?)
    }

    /// Connect to relays
    ///
    /// Attempts to initiate a connection for relays with
    /// [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`].
    /// A background connection task is spawned for each such relay, which then tries
    /// to establish the connection.
    /// Any relay not in one of these two statuses is skipped.
    ///
    /// For further details, see the documentation of [`Relay::connect`].
    ///
    /// [`RelayStatus::Initialized`]: crate::relay::RelayStatus::Initialized
    /// [`RelayStatus::Terminated`]: crate::relay::RelayStatus::Terminated
    #[inline]
    pub fn connect(&self) -> Connect {
        Connect::new(self)
    }

    /// Waits for relays connections
    ///
    /// Wait for relays connections at most for the specified `timeout`.
    /// The code continues when the relays are connected or the `timeout` is reached.
    #[deprecated(
        since = "0.45.0",
        note = "use `client.connect().and_wait(timeout).await` instead"
    )]
    pub async fn wait_for_connection(&self, timeout: Duration) {
        self.connect().and_wait(timeout).await;
    }

    /// Try to establish a connection with the relays.
    ///
    /// Attempts to establish a connection for every relay currently in
    /// [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`]
    /// without spawning the connection task if it fails.
    /// This means that if the connection fails, no automatic retries are scheduled.
    /// Use [`Client::connect`] if you want to immediately spawn a connection task,
    /// regardless of whether the initial connection succeeds.
    ///
    /// For further details, see the documentation of [`Relay::try_connect`].
    ///
    /// [`RelayStatus::Initialized`]: crate::relay::RelayStatus::Initialized
    /// [`RelayStatus::Terminated`]: crate::relay::RelayStatus::Terminated
    #[inline]
    pub fn try_connect(&self) -> TryConnect {
        TryConnect::new(self)
    }

    /// Disconnect from all relays
    #[inline]
    pub async fn disconnect(&self) {
        self.pool.disconnect().await
    }

    /// Get subscriptions
    #[inline]
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, HashMap<RelayUrl, Vec<Filter>>> {
        self.pool.subscriptions().await
    }

    /// Get subscription
    #[inline]
    pub async fn subscription(&self, id: &SubscriptionId) -> HashMap<RelayUrl, Vec<Filter>> {
        self.pool.subscription(id).await
    }

    /// Subscribe to events
    ///
    /// This method supports multiple subscription patterns through the [`ReqTarget`] type:
    /// - **Broadcast**: Send the same filters to all connected relays
    /// - **Targeted**: Send specific filters to specific relays
    ///
    /// The subscription will remain active until explicitly closed or until auto-close
    /// conditions are met (if configured with [`Subscribe::close_on`]).
    ///
    /// ## Gossip Support
    ///
    /// If `gossip` is enabled, events will be requested also from NIP-65 relays
    /// (automatically discovered) of public keys included in filters (if any).
    ///
    /// ## Auto-closing subscriptions
    ///
    /// It's possible to automatically close a subscription by configuring [`Subscribe::close_on`] (see below examples).
    ///
    /// **Note**: auto-closing subscriptions aren't saved in the subscriptions map!
    ///
    /// # Returns
    ///
    /// Returns a [`Subscribe`] builder that can be configured before execution.
    ///
    /// # Examples
    ///
    /// ## Broadcast to all relays
    ///
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # async fn example() -> Result<()> {
    /// let client = Client::default();
    ///
    /// // Add some relays
    /// client.add_relay("wss://relay1.example.com").await?;
    /// client.add_relay("wss://relay2.example.com").await?;
    ///
    /// // Subscribe with a single filter to all relays
    /// let filter = Filter::new().kind(Kind::TextNote).since(Timestamp::now());
    ///
    /// let output = client.subscribe(filter).await?;
    /// println!("Subscription ID: {}", output.val);
    /// println!("Successful relays: {:?}", output.success);
    /// println!("Failed relays: {:?}", output.failed);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Target specific relays
    ///
    /// ```rust,no_run
    /// use std::collections::HashMap;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # async fn example() -> Result<()> {
    /// let client = Client::default();
    ///
    /// // Add relays
    /// client.add_relay("wss://relay1.example.com").await?;
    /// client.add_relay("wss://relay2.example.com").await?;
    ///
    /// // Subscribe with different filters per relay
    /// let mut targets = HashMap::new();
    /// targets.insert(
    ///     "wss://relay1.example.com",
    ///     vec![Filter::new().kind(Kind::TextNote).limit(10)],
    /// );
    /// targets.insert(
    ///     "wss://relay2.example.com",
    ///     vec![Filter::new().kind(Kind::Metadata).limit(5)],
    /// );
    ///
    /// let output = client.subscribe(targets).await?;
    /// println!("Subscription ID: {}", output.val);
    /// println!("Successful relays: {:?}", output.success);
    /// println!("Failed relays: {:?}", output.failed);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## With custom ID and auto-close
    ///
    /// ```rust,no_run
    /// use std::time::Duration;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # async fn example() -> Result<()> {
    /// let client = Client::default();
    ///
    /// client.add_relay("wss://relay.example.com").await?;
    ///
    /// let filter = Filter::new().kind(Kind::TextNote).limit(10);
    /// let custom_id = SubscriptionId::generate();
    ///
    /// let auto_close =
    ///     SubscribeAutoCloseOptions::default().idle_timeout(Some(Duration::from_secs(60)));
    ///
    /// let output = client
    ///     .subscribe(filter)
    ///     .with_id(custom_id.clone())
    ///     .close_on(auto_close)
    ///     .await?;
    ///
    /// println!("Successful relays: {:?}", output.success);
    /// println!("Failed relays: {:?}", output.failed);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn subscribe<'client, 'url, F>(&'client self, target: F) -> Subscribe<'client, 'url>
    where
        F: Into<ReqTarget<'url>>,
    {
        Subscribe::new(self, target.into())
    }

    /// Unsubscribe from a REQ
    #[inline]
    pub fn unsubscribe<'id>(&self, id: &'id SubscriptionId) -> Unsubscribe<'_, 'id> {
        Unsubscribe::new(self, id)
    }

    /// Unsubscribe from all REQs
    #[inline]
    pub fn unsubscribe_all(&self) -> UnsubscribeAll {
        UnsubscribeAll::new(self)
    }

    /// Sync events with relays (negentropy reconciliation)
    ///
    /// If `gossip` is enabled the events will be reconciled also from
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// <https://github.com/hoytech/negentropy>
    #[inline]
    pub fn sync<'url>(&self, filter: Filter) -> SyncEvents<'_, 'url> {
        SyncEvents::new(self, filter)
    }

    /// Sync events with specific relays (negentropy reconciliation)
    ///
    /// <https://github.com/hoytech/negentropy>
    #[deprecated(
        since = "0.45.0",
        note = "use `client.sync(filter).with(urls).await` instead"
    )]
    pub async fn sync_with<'a, I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        self.sync(filter).with(urls).opts(opts.clone()).await
    }

    /// Fetch events from relays
    ///
    /// # Overview
    ///
    /// This is an **auto-closing subscription** and will be closed automatically on `EOSE`.
    /// For long-lived subscriptions, use [`Client::subscribe`].
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled, the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// # Example
    /// ```rust,no_run
    /// # use std::time::Duration;
    /// # use nostr_sdk::prelude::*;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #   let client = Client::default();
    /// let filter = Filter::new().kind(Kind::TextNote).limit(10);
    ///
    /// let events = client
    ///     .fetch_events(filter)
    ///     .timeout(Duration::from_secs(10))
    ///     .await?;
    /// # Ok(()) }
    /// ```
    #[inline]
    pub fn fetch_events<'client, 'url, F>(&'client self, target: F) -> FetchEvents<'client, 'url>
    where
        F: Into<ReqTarget<'url>>,
    {
        FetchEvents::new(self, target.into())
    }

    /// Stream events from relays
    ///
    /// # Overview
    ///
    /// This is an **auto-closing subscription** and will be closed automatically on `EOSE`.
    /// For long-lived subscriptions, use [`Client::subscribe`].
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled the events will be streamed also from
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    #[inline]
    pub fn stream_events<'client, 'url, F>(&'client self, target: F) -> StreamEvents<'client, 'url>
    where
        F: Into<ReqTarget<'url>>,
    {
        StreamEvents::new(self, target.into())
    }

    /// Send the client message to a **specific relays**
    #[inline]
    pub async fn send_msg_to<'a, I, U>(
        &self,
        urls: I,
        msg: ClientMessage<'_>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        self.batch_msg_to(urls, vec![msg]).await
    }

    /// Batch send client messages to **specific relays**
    #[inline]
    pub async fn batch_msg_to<'a, I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage<'_>>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        let mut set: HashSet<RelayUrl> = HashSet::new();

        for url in urls {
            let url: RelayUrlArg<'a> = url.into();
            let url: Cow<RelayUrl> = url.try_as_relay_url()?;
            set.insert(url.into_owned());
        }

        Ok(self.pool.batch_msg_to(set, msgs).await?)
    }

    /// Send the event to relays
    ///
    /// # Overview
    ///
    /// Send the [`Event`] to all relays with [`RelayCapabilities::WRITE`] flag.
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled:
    /// - the [`Event`] will be sent also to NIP65 relays (automatically discovered);
    /// - the gossip data will be updated, if the [`Event`] is a NIP17/NIP65 relay list.
    #[inline]
    pub fn send_event<'event, 'url>(&self, event: &'event Event) -> SendEvent<'_, 'event, 'url> {
        SendEvent::new(self, event)
    }

    /// Send event to specific relays
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled and the [`Event`] is a NIP17/NIP65 relay list,
    /// the gossip data will be updated.
    #[deprecated(
        since = "0.45.0",
        note = "use `client.send_event(event).to(urls).await` instead"
    )]
    pub async fn send_event_to<'a, I, U>(
        &self,
        urls: I,
        event: &Event,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        self.send_event(event).to(urls).await
    }

    /// Build, sign and return [`Event`]
    ///
    /// This method requires a [`NostrSigner`].
    pub async fn sign_event_builder(&self, builder: EventBuilder) -> Result<Event, Error> {
        let signer = self.signer().await?;
        Ok(builder.sign(&signer).await?)
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to relays.
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// Check [`Client::send_event`] from more details.
    #[inline]
    pub async fn send_event_builder(
        &self,
        builder: EventBuilder,
    ) -> Result<Output<EventId>, Error> {
        let event: Event = self.sign_event_builder(builder).await?;
        self.send_event(&event).await
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to specific relays.
    ///
    /// This method requires a [`NostrSigner`].
    #[inline]
    pub async fn send_event_builder_to<'a, I, U>(
        &self,
        urls: I,
        builder: EventBuilder,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: Into<RelayUrlArg<'a>>,
    {
        let event: Event = self.sign_event_builder(builder).await?;
        self.send_event(&event).to(urls).await
    }

    /// Handle notifications
    ///
    /// The closure function expects a `bool` as output: return `true` to exit from the notification loop.
    pub async fn handle_notifications<F, Fut>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(ClientNotification) -> Fut,
        Fut: Future<Output = Result<bool>>,
    {
        let mut notifications = self.notifications();
        while let Ok(notification) = notifications.recv().await {
            let shutdown: bool = ClientNotification::Shutdown == notification;
            let exit: bool = func(notification)
                .await
                .map_err(|e| Error::Handler(e.to_string()))?;
            if exit || shutdown {
                break;
            }
        }
        Ok(())
    }
}

// Gossip
impl Client {
    async fn check_outdated_public_keys<'a, I>(
        &self,
        gossip: &Arc<dyn NostrGossip>,
        public_keys: I,
        gossip_kind: GossipListKind,
    ) -> Result<HashSet<PublicKey>, Error>
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        // First check: check if there are outdated public keys.
        let mut outdated_public_keys: HashSet<PublicKey> = HashSet::new();

        for public_key in public_keys.into_iter() {
            // Get the public key status
            let status = gossip.status(public_key, gossip_kind).await?;

            if let GossipPublicKeyStatus::Outdated { .. } = status {
                outdated_public_keys.insert(*public_key);
            }
        }

        Ok(outdated_public_keys)
    }

    /// Check for and update outdated public key data
    ///
    /// Steps:
    /// 1. Attempts negentropy sync with DISCOVERY and READ relays to efficiently reconcile data
    /// 2. For any relays where negentropy sync fails, falls back to standard REQ messages to fetch the gossip lists
    async fn check_and_update_gossip<I>(
        &self,
        gossip: &GossipWrapper,
        public_keys: I,
        gossip_kind: GossipListKind,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let public_keys: HashSet<PublicKey> = public_keys.into_iter().collect();

        // First check: check if there are outdated public keys.
        let outdated_public_keys_first_check: HashSet<PublicKey> = self
            .check_outdated_public_keys(gossip, public_keys.iter(), gossip_kind)
            .await?;

        // No outdated public keys, immediately return.
        if outdated_public_keys_first_check.is_empty() {
            tracing::debug!(kind = ?gossip_kind, "Gossip data is up to date.");
            return Ok(());
        }

        let sync_id: u64 = gossip.next_sync_id();

        tracing::debug!(sync_id, "Acquiring gossip sync permit...");

        let _permit = self.gossip_sync.acquire().await;

        tracing::debug!(sync_id, kind = ?gossip_kind, "Acquired gossip sync permit. Start syncing...");

        // Second check: check data is still outdated after acquiring permit
        // (another process might have updated it while we were waiting)
        let outdated_public_keys: HashSet<PublicKey> = self
            .check_outdated_public_keys(gossip, public_keys.iter(), gossip_kind)
            .await?;

        // Double-check: data might have been updated while waiting for permit
        if outdated_public_keys.is_empty() {
            tracing::debug!(sync_id = %sync_id, kind = ?gossip_kind, "Gossip sync skipped: data updated by another process while acquiring permits.");
            return Ok(());
        }

        // Negentropy sync and database check
        let (output, stored_events) = self
            .check_and_update_gossip_sync(
                sync_id,
                gossip,
                &gossip_kind,
                outdated_public_keys.clone(),
            )
            .await?;

        // Keep track of the missing public keys
        let mut missing_public_keys: HashSet<PublicKey> = outdated_public_keys;

        // Check if sync failed for some relay
        if !output.failed.is_empty() {
            tracing::debug!(sync_id,
                relays = ?output.failed,
                "Gossip sync failed for some relays."
            );

            // Try to fetch the updated events
            self.check_and_update_gossip_fetch(
                sync_id,
                gossip,
                &gossip_kind,
                &output,
                &stored_events,
                &mut missing_public_keys,
            )
            .await?;

            // Get the missing events
            if !missing_public_keys.is_empty() {
                // Try to fetch the missing events
                self.check_and_update_gossip_missing(
                    sync_id,
                    gossip,
                    &gossip_kind,
                    &output,
                    missing_public_keys,
                )
                .await?;
            }
        }

        tracing::debug!(sync_id, kind = ?gossip_kind, "Gossip sync terminated.");

        Ok(())
    }

    /// Check and update gossip data using negentropy sync
    async fn check_and_update_gossip_sync(
        &self,
        sync_id: u64,
        gossip: &Arc<dyn NostrGossip>,
        gossip_kind: &GossipListKind,
        outdated_public_keys: HashSet<PublicKey>,
    ) -> Result<(Output<Reconciliation>, Events), Error> {
        // Get kind
        let kind: Kind = gossip_kind.to_event_kind();

        tracing::debug!(
            sync_id,
            public_keys = outdated_public_keys.len(),
            "Syncing outdated gossip data."
        );

        // Compose database filter
        let filter: Filter = Filter::default().authors(outdated_public_keys).kind(kind);

        // Get DISCOVERY and READ relays
        let urls: HashSet<RelayUrl> = self
            .pool
            .relay_urls_with_any_cap(RelayCapabilities::DISCOVERY | RelayCapabilities::READ)
            .await;

        // Negentropy sync
        // NOTE: the received events are automatically processed in the middleware!
        let opts: SyncOptions = SyncOptions::default().direction(SyncDirection::Down);
        let output: Output<Reconciliation> =
            self.sync(filter.clone()).with(urls).opts(opts).await?;

        // Get events from the database
        let stored_events: Events = self.database().query(filter).await?;

        // Process stored events
        for event in stored_events.iter() {
            // Update the last check for this public key
            gossip
                .update_fetch_attempt(&event.pubkey, *gossip_kind)
                .await?;

            // Skip events that has already processed in the middleware
            if output.received.contains(&event.id) {
                continue;
            }

            gossip.process(event, None).await?;
        }

        Ok((output, stored_events))
    }

    /// Try to fetch the new gossip events from the relays that failed the negentropy sync
    async fn check_and_update_gossip_fetch(
        &self,
        sync_id: u64,
        gossip: &Arc<dyn NostrGossip>,
        gossip_kind: &GossipListKind,
        output: &Output<Reconciliation>,
        stored_events: &Events,
        missing_public_keys: &mut HashSet<PublicKey>,
    ) -> Result<(), Error> {
        // Get kind
        let kind: Kind = gossip_kind.to_event_kind();

        let mut filters: Vec<Filter> = Vec::new();

        let skip_ids: HashSet<EventId> = output.local.union(&output.received).copied().collect();

        // Try to fetch from relays only the newer events (last created_at + 1)
        for event in stored_events.iter() {
            // Remove from the missing set
            missing_public_keys.remove(&event.pubkey);

            // Skip the already synced events
            if skip_ids.contains(&event.id) {
                continue;
            }

            // Construct filter
            let filter: Filter = Filter::new()
                .author(event.pubkey)
                .kind(kind)
                .since(event.created_at + Duration::from_secs(1))
                .limit(1);

            filters.push(filter);
        }

        if filters.is_empty() {
            tracing::debug!(
                sync_id,
                "Skipping gossip fetch, as it's no longer required."
            );
            return Ok(());
        }

        tracing::debug!(
            sync_id,
            filters = filters.len(),
            "Fetching outdated gossip data from relays."
        );

        // Split filters in chunks of 10
        for chunk in filters.chunks(10) {
            // Fetch the events
            // NOTE: the received events are automatically processed in the middleware!

            let mut targets = HashMap::with_capacity(output.failed.len());

            for url in output.failed.keys() {
                targets.insert(url.clone(), chunk.to_vec());
            }

            let mut stream = self
                .pool
                .stream_events(
                    targets,
                    Some(Duration::from_secs(10)),
                    ReqExitPolicy::ExitOnEOSE,
                )
                .await?;

            // Update the last check for the fetched public keys
            while let Some((url, event)) = stream.next().await {
                match event {
                    Ok(event) => {
                        // Update the last check for this public key
                        gossip
                            .update_fetch_attempt(&event.pubkey, *gossip_kind)
                            .await?;
                    }
                    Err(e) => {
                        tracing::error!(%url, error = %e, "Failed to fetch outdated gossip data from relay.");
                    }
                }
            }
        }

        Ok(())
    }

    /// Try to fetch the gossip events for the missing public keys from the relays that failed the negentropy sync
    async fn check_and_update_gossip_missing(
        &self,
        sync_id: u64,
        gossip: &Arc<dyn NostrGossip>,
        gossip_kind: &GossipListKind,
        output: &Output<Reconciliation>,
        missing_public_keys: HashSet<PublicKey>,
    ) -> Result<(), Error> {
        // Get kind
        let kind: Kind = gossip_kind.to_event_kind();

        tracing::debug!(
            sync_id,
            public_keys = missing_public_keys.len(),
            "Fetching missing gossip data from relays."
        );

        let missing_filter: Filter = Filter::default()
            .authors(missing_public_keys.clone())
            .kind(kind);

        let mut targets = HashMap::with_capacity(output.failed.len());

        for url in output.failed.keys() {
            targets.insert(url.clone(), vec![missing_filter.clone()]);
        }

        // NOTE: the received events are automatically processed in the middleware!
        let mut stream = self
            .pool
            .stream_events(
                targets,
                Some(Duration::from_secs(10)),
                ReqExitPolicy::ExitOnEOSE,
            )
            .await?;

        // Consume the stream
        #[allow(clippy::redundant_pattern_matching)]
        while let Some(..) = stream.next().await {}

        // Update the last check for the missing public keys
        for pk in missing_public_keys.into_iter() {
            gossip.update_fetch_attempt(&pk, *gossip_kind).await?;
        }

        Ok(())
    }

    /// Break down filters for gossip and discovery relays
    async fn break_down_filter(
        &self,
        gossip: &GossipWrapper,
        filter: Filter,
    ) -> Result<HashMap<RelayUrl, Filter>, Error> {
        // Extract all public keys from filters
        let public_keys = filter.extract_public_keys();

        // Find pattern to decide what list to update
        let pattern: GossipFilterPattern = gossip::find_filter_pattern(&filter);

        // Update outdated public keys
        match &pattern {
            GossipFilterPattern::Nip65 => {
                self.check_and_update_gossip(gossip, public_keys, GossipListKind::Nip65)
                    .await?;
            }
            GossipFilterPattern::Nip65AndNip17 => {
                self.check_and_update_gossip(
                    gossip,
                    public_keys.iter().copied(),
                    GossipListKind::Nip65,
                )
                .await?;
                self.check_and_update_gossip(gossip, public_keys, GossipListKind::Nip17)
                    .await?;
            }
        }

        // Broken-down filters
        let filters: HashMap<RelayUrl, Filter> = match gossip
            .break_down_filter(
                filter,
                pattern,
                &self.opts.gossip.limits,
                self.opts.gossip.allowed,
            )
            .await?
        {
            BrokenDownFilters::Filters(filters) => filters,
            BrokenDownFilters::Orphan(filter) | BrokenDownFilters::Other(filter) => {
                // Get read relays
                let read_relays: HashSet<RelayUrl> = self.pool.read_relay_urls().await;

                let mut map = HashMap::with_capacity(read_relays.len());
                for url in read_relays.into_iter() {
                    map.insert(url, filter.clone());
                }
                map
            }
        };

        // Add gossip (outbox and inbox) relays
        for url in filters.keys() {
            self.add_relay(url)
                .capabilities(RelayCapabilities::GOSSIP)
                .and_connect()
                .await?;
        }

        // Check if filters are empty
        // TODO: this can't be empty, right?
        if filters.is_empty() {
            return Err(Error::GossipFiltersEmpty);
        }

        Ok(filters)
    }

    /// Break down filters for gossip and discovery relays
    async fn break_down_filters<F>(
        &self,
        gossip: &GossipWrapper,
        filters: F,
    ) -> Result<HashMap<RelayUrl, Vec<Filter>>, Error>
    where
        F: Into<Vec<Filter>>,
    {
        let filters: Vec<Filter> = filters.into();

        let mut output: HashMap<RelayUrl, HashSet<Filter>> = HashMap::new();

        for filter in filters {
            let f = self.break_down_filter(gossip, filter).await?;

            for (url, filter) in f {
                output.entry(url).or_default().insert(filter);
            }
        }

        // TODO: avoid this and returns the HashSet. At the moment this is required due to the Into<Vec<Filter>>
        Ok(output
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use nostr_relay_builder::MockRelay;

    use super::*;
    use crate::pool;
    use crate::relay::RelayStatus;

    #[tokio::test]
    async fn test_shutdown() {
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let client = Client::default();

        client.add_relay(&url).await.unwrap();

        client.connect().await;

        assert!(!client.is_shutdown());

        tokio::time::sleep(Duration::from_secs(1)).await;

        client.shutdown().await;

        // All relays must be removed
        assert!(client.relays().all().await.is_empty());

        // Client must be marked as shutdown
        assert!(client.is_shutdown());

        assert!(matches!(
            client.add_relay(url).await.unwrap_err(),
            Error::RelayPool(pool::Error::Shutdown)
        ));
    }

    #[tokio::test]
    async fn test_shutdown_on_drop() {
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let relay: Relay = {
            let client: Client = Client::default();

            client.add_relay(&url).and_connect().await.unwrap();

            assert!(!client.is_shutdown());

            tokio::time::sleep(Duration::from_millis(500)).await;

            let relay = client.relay(&url).await.unwrap().unwrap();

            assert!(relay.is_connected());

            relay
        };
        // Client is dropped here

        tokio::time::sleep(Duration::from_secs(1)).await;

        // When the client is dropped, all relays are shutdown
        assert_eq!(relay.status(), RelayStatus::Terminated);
    }
}
