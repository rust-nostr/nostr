// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Client

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::iter;
use std::sync::Arc;
use std::time::Duration;

use nostr::prelude::*;
use nostr_database::prelude::*;
use nostr_relay_pool::prelude::*;
use tokio::sync::broadcast;

pub mod builder;
mod error;
pub mod options;

pub use self::builder::ClientBuilder;
pub use self::error::Error;
pub use self::options::Options;
#[cfg(not(target_arch = "wasm32"))]
pub use self::options::{Connection, ConnectionTarget};
use crate::gossip::{BrokenDownFilters, Gossip};

/// Nostr client
#[derive(Debug, Clone)]
pub struct Client {
    pool: RelayPool,
    gossip: Gossip,
    opts: Options,
}

impl Default for Client {
    #[inline]
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Client {
    /// Construct client with signer
    ///
    /// To construct a client without signer use [`Client::default`].
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// let keys = Keys::generate();
    /// let client = Client::new(keys);
    /// ```
    #[inline]
    pub fn new<T>(signer: T) -> Self
    where
        T: IntoNostrSigner,
    {
        Self::builder().signer(signer).build()
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
    /// let opts = Options::default().gossip(true);
    /// let client: Client = Client::builder().signer(signer).opts(opts).build();
    /// ```
    #[inline]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    fn from_builder(builder: ClientBuilder) -> Self {
        // Construct relay pool builder
        let pool_builder: RelayPoolBuilder = RelayPoolBuilder {
            websocket_transport: builder.websocket_transport,
            admit_policy: builder.admit_policy,
            monitor: builder.monitor,
            opts: builder.opts.pool,
            __database: builder.database,
            __signer: builder.signer,
        };

        // Construct client
        Self {
            pool: pool_builder.build(),
            gossip: Gossip::new(),
            opts: builder.opts,
        }
    }

    /// Update minimum POW difficulty for received events
    ///
    /// Events with a POW lower than the current value will be ignored to prevent resources exhaustion.
    #[deprecated(
        since = "0.40.0",
        note = "This no longer works, please use `AdmitPolicy` instead."
    )]
    pub fn update_min_pow_difficulty(&self, _difficulty: u8) {}

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

    /// Get [`RelayPool`]
    #[inline]
    pub fn pool(&self) -> &RelayPool {
        &self.pool
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
    /// * reset [`Options`]
    /// * remove the database
    /// * clear the gossip graph
    pub async fn reset(&self) {
        self.unsubscribe_all().await;
        self.force_remove_all_relays().await;
        self.unset_signer().await;
    }

    /// Completely shutdown client
    #[inline]
    pub async fn shutdown(&self) {
        self.pool.shutdown().await
    }

    /// Get new notification listener
    ///
    /// <div class="warning">When you call this method, you subscribe to the notifications channel from that precise moment. Anything received by relay/s before that moment is not included in the channel!</div>
    #[inline]
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.pool.notifications()
    }

    /// Get relays with [`RelayServiceFlags::READ`] or [`RelayServiceFlags::WRITE`] flags
    ///
    /// Call [`RelayPool::all_relays`] to get all relays
    /// or [`RelayPool::relays_with_flag`] to get relays with specific [`RelayServiceFlags`].
    #[inline]
    pub async fn relays(&self) -> HashMap<RelayUrl, Relay> {
        self.pool.relays().await
    }

    /// Get a previously added [`Relay`]
    #[inline]
    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.relay(url).await?)
    }

    async fn compose_relay_opts(&self, _url: &RelayUrl) -> RelayOptions {
        let opts: RelayOptions = RelayOptions::new();

        // Set connection mode
        #[cfg(not(target_arch = "wasm32"))]
        let opts: RelayOptions = match &self.opts.connection.mode {
            ConnectionMode::Direct => opts,
            ConnectionMode::Proxy(..) => match self.opts.connection.target {
                ConnectionTarget::All => opts.connection_mode(self.opts.connection.mode.clone()),
                ConnectionTarget::Onion => {
                    if _url.is_onion() {
                        opts.connection_mode(self.opts.connection.mode.clone())
                    } else {
                        opts
                    }
                }
            },
            #[cfg(feature = "tor")]
            ConnectionMode::Tor { .. } => match self.opts.connection.target {
                ConnectionTarget::All => opts.connection_mode(self.opts.connection.mode.clone()),
                ConnectionTarget::Onion => {
                    if _url.is_onion() {
                        opts.connection_mode(self.opts.connection.mode.clone())
                    } else {
                        opts
                    }
                }
            },
        };

        // Set limits
        opts.limits(self.opts.relay_limits.clone())
            .max_avg_latency(self.opts.max_avg_latency)
    }

    /// If return `false` means that already existed
    async fn get_or_add_relay_with_flag<U>(
        &self,
        url: U,
        flag: RelayServiceFlags,
    ) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert into url
        let url: RelayUrl = url.try_into_url().map_err(pool::Error::from)?;

        // Compose relay options
        let opts: RelayOptions = self.compose_relay_opts(&url).await;

        // Set flag
        let opts: RelayOptions = opts.flags(flag);

        // Add relay with opts or edit current one
        // TODO: remove clone here
        match self.pool.__get_or_add_relay(url.clone(), opts).await? {
            Some(relay) => {
                relay.flags().add(flag);
                Ok(false)
            }
            None => {
                // TODO: move autoconnect to `Relay`?
                // Connect if `autoconnect` is enabled
                if self.opts.autoconnect {
                    self.connect_relay::<RelayUrl>(url).await?;
                }

                Ok(true)
            }
        }
    }

    /// Add relay
    ///
    /// Relays added with this method will have both [`RelayServiceFlags::READ`] and [`RelayServiceFlags::WRITE`] flags enabled.
    ///
    /// If the relay already exists, the flags will be updated and `false` returned.
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use [`Client::subscribe_to`] method instead of [`Client::subscribe`],
    /// to avoid to set pool subscriptions.
    ///
    /// This method use previously set or default [`Options`] to configure the [`Relay`] (ex. set proxy, set min POW, set relay limits, ...).
    /// To use custom [`RelayOptions`] use [`RelayPool::add_relay`].
    ///
    /// Connection is **NOT** automatically started with relay, remember to call [`Client::connect`]!
    #[inline]
    pub async fn add_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.get_or_add_relay_with_flag(url, RelayServiceFlags::default())
            .await
    }

    /// Add discovery relay
    ///
    /// If relay already exists, this method automatically add the [`RelayServiceFlags::DISCOVERY`] flag to it and return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[inline]
    pub async fn add_discovery_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.get_or_add_relay_with_flag(url, RelayServiceFlags::PING | RelayServiceFlags::DISCOVERY)
            .await
    }

    /// Add read relay
    ///
    /// If relay already exists, this method add the [`RelayServiceFlags::READ`] flag to it and return `false`.
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    #[inline]
    pub async fn add_read_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.get_or_add_relay_with_flag(url, RelayServiceFlags::PING | RelayServiceFlags::READ)
            .await
    }

    /// Add write relay
    ///
    /// If relay already exists, this method add the [`RelayServiceFlags::WRITE`] flag to it and return `false`.
    #[inline]
    pub async fn add_write_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.get_or_add_relay_with_flag(url, RelayServiceFlags::PING | RelayServiceFlags::WRITE)
            .await
    }

    #[inline]
    async fn add_gossip_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.get_or_add_relay_with_flag(url, RelayServiceFlags::PING | RelayServiceFlags::GOSSIP)
            .await
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has [`RelayServiceFlags::GOSSIP`], it will not be removed from the pool and its
    /// flags will be updated (remove [`RelayServiceFlags::READ`],
    /// [`RelayServiceFlags::WRITE`] and [`RelayServiceFlags::DISCOVERY`] flags).
    ///
    /// To force remove the relay, use [`Client::force_remove_relay`].
    #[inline]
    pub async fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.remove_relay(url).await?)
    }

    /// Force remove and disconnect relay
    ///
    /// Note: this method will remove the relay, also if it's in use for the gossip model or other service!
    #[inline]
    pub async fn force_remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.force_remove_relay(url).await?)
    }

    /// Disconnect and remove all relays
    ///
    /// Some relays used by some services could not be disconnected with this method
    /// (like the ones used for gossip).
    /// Use [`Client::force_remove_all_relays`] to remove every relay.
    #[inline]
    pub async fn remove_all_relays(&self) {
        self.pool.remove_all_relays().await
    }

    /// Disconnect and force remove all relays
    #[inline]
    pub async fn force_remove_all_relays(&self) {
        self.pool.force_remove_all_relays().await
    }

    /// Connect to a previously added relay
    ///
    /// Check [`RelayPool::connect_relay`] docs to learn more.
    #[inline]
    pub async fn connect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.connect_relay(url).await?)
    }

    /// Try to connect to a previously added relay
    ///
    /// For further details, see the documentation of [`RelayPool::try_connect_relay`].
    #[inline]
    pub async fn try_connect_relay<U>(&self, url: U, timeout: Duration) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.try_connect_relay(url, timeout).await?)
    }

    /// Disconnect relay
    #[inline]
    pub async fn disconnect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.disconnect_relay(url).await?)
    }

    /// Connect to all added relays
    ///
    /// Attempts to initiate a connection for every relay currently in
    /// [`RelayStatus::Initialized`] or [`RelayStatus::Terminated`].
    /// A background connection task is spawned for each such relay, which then tries
    /// to establish the connection.
    /// Any relay not in one of these two statuses is skipped.
    ///
    /// For further details, see the documentation of [`Relay::connect`].
    #[inline]
    pub async fn connect(&self) {
        self.pool.connect().await;
    }

    /// Waits for relays connections
    ///
    /// Wait for relays connections at most for the specified `timeout`.
    /// The code continues when the relays are connected or the `timeout` is reached.
    #[inline]
    pub async fn wait_for_connection(&self, timeout: Duration) {
        self.pool.wait_for_connection(timeout).await
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
    #[inline]
    pub async fn try_connect(&self, timeout: Duration) -> Output<()> {
        self.pool.try_connect(timeout).await
    }

    /// Connect to all added relays
    ///
    /// Try to connect to the relays and wait for them to be connected at most for the specified `timeout`.
    /// The code continues if the `timeout` is reached or if all relays connect.
    #[deprecated(
        since = "0.39.0",
        note = "Use `connect` + `wait_for_connection` instead."
    )]
    pub async fn connect_with_timeout(&self, timeout: Duration) {
        self.pool.try_connect(timeout).await;
    }

    /// Disconnect from all relays
    #[inline]
    pub async fn disconnect(&self) {
        self.pool.disconnect().await
    }

    /// Get pool subscriptions
    #[inline]
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Filter> {
        self.pool.subscriptions().await
    }

    /// Get pool subscription
    #[inline]
    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Filter> {
        self.pool.subscription(id).await
    }

    /// Subscribe to filters
    ///
    /// This method create a new subscription. None of the previous subscriptions will be edited/closed when you call this!
    /// So remember to unsubscribe when you no longer need it. You can get all your active **pool** (non-auto-closing) subscriptions
    /// by calling `client.subscriptions().await`.
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// # Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeAutoCloseOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::prelude::*;
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// #   let keys = Keys::generate();
    /// #   let client = Client::new(keys.clone());
    /// // Compose filter
    /// let subscription = Filter::new()
    ///     .pubkeys(vec![keys.public_key()])
    ///     .since(Timestamp::now());
    ///
    /// // Subscribe
    /// let output = client.subscribe(subscription, None).await?;
    /// println!("Subscription ID: {}", output.val);
    ///
    /// // Auto-closing subscription
    /// let id = SubscriptionId::generate();
    /// let subscription = Filter::new().kind(Kind::TextNote).limit(10);
    /// let opts = SubscribeAutoCloseOptions::default().exit_policy(ReqExitPolicy::ExitOnEOSE);
    /// let output = client.subscribe(subscription, Some(opts)).await?;
    /// println!("Subscription ID: {} [auto-closing]", output.val);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(
        &self,
        filter: Filter,
        opts: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<SubscriptionId>, Error> {
        let id: SubscriptionId = SubscriptionId::generate();
        let output: Output<()> = self.subscribe_with_id(id.clone(), filter, opts).await?;
        Ok(Output {
            val: id,
            success: output.success,
            failed: output.failed,
        })
    }

    /// Subscribe to filters with custom [SubscriptionId]
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// # Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeAutoCloseOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filter: Filter,
        opts: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<()>, Error> {
        let opts: SubscribeOptions = SubscribeOptions::default().close_on(opts);

        if self.opts.gossip {
            self.gossip_subscribe(id, filter, opts).await
        } else {
            Ok(self.pool.subscribe_with_id(id, filter, opts).await?)
        }
    }

    /// Subscribe to filters to specific relays
    ///
    /// This method create a new subscription. None of the previous subscriptions will be edited/closed when you call this!
    /// So remember to unsubscribe when you no longer need it.
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeAutoCloseOptions].
    #[inline]
    pub async fn subscribe_to<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<SubscriptionId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let opts: SubscribeOptions = SubscribeOptions::default().close_on(opts);
        Ok(self.pool.subscribe_to(urls, filter, opts).await?)
    }

    /// Subscribe to filter with custom [SubscriptionId] to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeAutoCloseOptions].
    #[inline]
    pub async fn subscribe_with_id_to<I, U>(
        &self,
        urls: I,
        id: SubscriptionId,
        filter: Filter,
        opts: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let opts: SubscribeOptions = SubscribeOptions::default().close_on(opts);
        Ok(self
            .pool
            .subscribe_with_id_to(urls, id, filter, opts)
            .await?)
    }

    /// Targeted subscription
    ///
    /// Subscribe to specific relays with specific filters
    #[inline]
    pub async fn subscribe_targeted<I, U>(
        &self,
        id: SubscriptionId,
        targets: I,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = (U, Filter)>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.subscribe_targeted(id, targets, opts).await?)
    }

    /// Unsubscribe
    #[inline]
    pub async fn unsubscribe(&self, id: &SubscriptionId) {
        self.pool.unsubscribe(id).await;
    }

    /// Unsubscribe from all subscriptions
    #[inline]
    pub async fn unsubscribe_all(&self) {
        self.pool.unsubscribe_all().await;
    }

    /// Sync events with relays (negentropy reconciliation)
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be reconciled also from
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// <https://github.com/hoytech/negentropy>
    #[inline]
    pub async fn sync(
        &self,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        if self.opts.gossip {
            return self.gossip_sync_negentropy(filter, opts).await;
        }

        Ok(self.pool.sync(filter, opts).await?)
    }

    /// Sync events with specific relays (negentropy reconciliation)
    ///
    /// <https://github.com/hoytech/negentropy>
    pub async fn sync_with<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.sync_with(urls, filter, opts).await?)
    }

    /// Fetch events from relays
    ///
    /// # Overview
    ///
    /// This is an **auto-closing subscription** and will be closed automatically on `EOSE`.
    /// To use another exit policy, check [`RelayPool::fetch_events`].
    /// For long-lived subscriptions, check [`Client::subscribe`].
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// # Example
    /// ```rust,no_run
    /// # use std::time::Duration;
    /// # use nostr_sdk::prelude::*;
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let keys = Keys::generate();
    /// #   let client = Client::new(keys.clone());
    /// let subscription = Filter::new()
    ///     .pubkeys(vec![keys.public_key()])
    ///     .since(Timestamp::now());
    ///
    /// let _events = client
    ///     .fetch_events(subscription, Duration::from_secs(10))
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn fetch_events(&self, filter: Filter, timeout: Duration) -> Result<Events, Error> {
        if self.opts.gossip {
            return self
                .gossip_fetch_events(filter, timeout, ReqExitPolicy::ExitOnEOSE)
                .await;
        }

        Ok(self
            .pool
            .fetch_events(filter, timeout, ReqExitPolicy::ExitOnEOSE)
            .await?)
    }

    /// Fetch events from specific relays
    ///
    /// # Overview
    ///
    /// This is an **auto-closing subscription** and will be closed automatically on `EOSE`.
    /// To use another exit policy, check [`RelayPool::fetch_events_from`].
    /// For long-lived subscriptions, check [`Client::subscribe_to`].
    #[inline]
    pub async fn fetch_events_from<I, U>(
        &self,
        urls: I,
        filter: Filter,
        timeout: Duration,
    ) -> Result<Events, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self
            .pool
            .fetch_events_from(urls, filter, timeout, ReqExitPolicy::ExitOnEOSE)
            .await?)
    }

    /// Get events both from database and relays
    ///
    /// # Overview
    ///
    /// This is an **auto-closing subscription** and will be closed automatically on `EOSE`.
    /// For long-lived subscriptions, check [`Client::subscribe`].
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// # Notes and alternative example
    ///
    /// This method will be deprecated in the future!
    /// This is a temporary solution for who still want to query events both from database and relays and merge the result.
    /// The optimal solution is to execute a [`Client::sync`] to reconcile missing events, [`Client::subscribe`] to get all
    /// new future events, [`NostrEventsDatabase::query`] to query stored events and [`Client::handle_notifications`] to listen-for/handle new events (i.e. to know when update the UI).
    /// This will allow very fast queries, low bandwidth usage (depending on how many events the client have to reconcile) and a lower load on the relays.
    ///
    /// You can obtain the same result with:
    /// ```rust,no_run
    /// # use std::time::Duration;
    /// # use nostr_sdk::prelude::*;
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let client = Client::default();
    /// # let filter = Filter::new().limit(1);
    /// // Query database
    /// let stored_events: Events = client.database().query(filter.clone()).await?;
    ///
    /// // Query relays
    /// let fetched_events: Events = client.fetch_events(filter, Duration::from_secs(10)).await?;
    ///
    /// // Merge result
    /// let events: Events = stored_events.merge(fetched_events);
    ///
    /// // Iter and print result
    /// for event in events.into_iter() {
    ///     println!("{}", event.as_json());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_combined_events(
        &self,
        filter: Filter,
        timeout: Duration,
    ) -> Result<Events, Error> {
        // Query database
        let stored_events: Events = self.database().query(filter.clone()).await?;

        // Query relays
        let fetched_events: Events = self.fetch_events(filter, timeout).await?;

        // Merge result
        Ok(stored_events.merge(fetched_events))
    }

    /// Stream events from relays
    ///
    /// # Overview
    ///
    /// This is an **auto-closing subscription** and will be closed automatically on `EOSE`.
    /// To use another exit policy, check [`RelayPool::stream_events`].
    /// For long-lived subscriptions, check [`Client::subscribe`].
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be streamed also from
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    pub async fn stream_events(
        &self,
        filter: Filter,
        timeout: Duration,
    ) -> Result<ReceiverStream<Event>, Error> {
        // Check if gossip is enabled
        if self.opts.gossip {
            self.gossip_stream_events(filter, timeout, ReqExitPolicy::ExitOnEOSE)
                .await
        } else {
            Ok(self
                .pool
                .stream_events(filter, timeout, ReqExitPolicy::ExitOnEOSE)
                .await?)
        }
    }

    /// Stream events from specific relays
    ///
    /// # Overview
    ///
    /// This is an **auto-closing subscription** and will be closed automatically on `EOSE`.
    /// To use another exit policy, check [`RelayPool::stream_events_from`].
    /// For long-lived subscriptions, check [`Client::subscribe_to`].
    #[inline]
    pub async fn stream_events_from<I, U>(
        &self,
        urls: I,
        filter: Filter,
        timeout: Duration,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self
            .pool
            .stream_events_from(urls, filter, timeout, ReqExitPolicy::default())
            .await?)
    }

    /// Stream events from specific relays with specific filters
    ///
    /// # Overview
    ///
    /// This is an **auto-closing subscription** and will be closed automatically on `EOSE`.
    /// To use another exit policy, check [`RelayPool::stream_events_targeted`].
    /// For long-lived subscriptions, check [`Client::subscribe_targeted`].
    pub async fn stream_events_targeted(
        &self,
        targets: HashMap<RelayUrl, Filter>,
        timeout: Duration,
    ) -> Result<ReceiverStream<Event>, Error> {
        Ok(self
            .pool
            .stream_events_targeted(targets, timeout, ReqExitPolicy::default())
            .await?)
    }

    /// Send the client message to a **specific relays**
    #[inline]
    pub async fn send_msg_to<I, U>(
        &self,
        urls: I,
        msg: ClientMessage<'_>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.send_msg_to(urls, msg).await?)
    }

    /// Batch send client messages to **specific relays**
    #[inline]
    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage<'_>>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.batch_msg_to(urls, msgs).await?)
    }

    /// Send the event to relays
    ///
    /// # Overview
    ///
    /// Send the [`Event`] to all relays with [`RelayServiceFlags::WRITE`] flag.
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]):
    /// - the [`Event`] will be sent also to NIP65 relays (automatically discovered);
    /// - the gossip data will be updated, if the [`Event`] is a NIP17/NIP65 relay list.
    #[inline]
    pub async fn send_event(&self, event: &Event) -> Result<Output<EventId>, Error> {
        // NOT gossip, send event to all relays
        if !self.opts.gossip {
            return Ok(self.pool.send_event(event).await?);
        }

        // Update gossip graph
        self.gossip.process_event(event).await;

        // Send event using gossip
        self.gossip_send_event(event, false).await
    }

    /// Send event to specific relays
    ///
    /// # Gossip
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) and the [`Event`] is a NIP17/NIP65 relay list,
    /// the gossip data will be updated.
    #[inline]
    pub async fn send_event_to<I, U>(
        &self,
        urls: I,
        event: &Event,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        // If gossip is enabled, update the gossip graph
        if self.opts.gossip {
            self.gossip.process_event(event).await;
        }

        // Send event to relays
        Ok(self.pool.send_event_to(urls, event).await?)
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
    ///
    /// Check [`Client::send_event_to`] from more details.
    #[inline]
    pub async fn send_event_builder_to<I, U>(
        &self,
        urls: I,
        builder: EventBuilder,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let event: Event = self.sign_event_builder(builder).await?;
        self.send_event_to(urls, &event).await
    }

    /// Fetch the newest public key metadata from relays.
    ///
    /// Returns [`None`] if the [`Metadata`] of the  [`PublicKey`] has not been found.
    ///
    /// Check [`Client::fetch_events`] for more details.
    ///
    /// If you only want to consult stored data,
    /// consider `client.database().profile(PUBKEY)`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub async fn fetch_metadata(
        &self,
        public_key: PublicKey,
        timeout: Duration,
    ) -> Result<Option<Metadata>, Error> {
        let filter: Filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        let events: Events = self.fetch_events(filter, timeout).await?;
        match events.first() {
            Some(event) => Ok(Some(Metadata::try_from(event)?)),
            None => Ok(None),
        }
    }

    /// Update metadata
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// # use nostr_sdk::prelude::*;
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let keys = Keys::generate();
    /// #   let client = Client::new(keys);
    /// let metadata = Metadata::new()
    ///     .name("username")
    ///     .display_name("My Username")
    ///     .about("Description")
    ///     .picture(Url::parse("https://example.com/avatar.png").unwrap())
    ///     .nip05("username@example.com");
    ///
    /// client.set_metadata(&metadata).await.unwrap();
    /// # }
    /// ```
    #[inline]
    pub async fn set_metadata(&self, metadata: &Metadata) -> Result<Output<EventId>, Error> {
        let builder = EventBuilder::metadata(metadata);
        self.send_event_builder(builder).await
    }

    async fn get_contact_list_filter(&self) -> Result<Filter, Error> {
        let signer = self.signer().await?;
        let public_key = signer.get_public_key().await?;
        let filter: Filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        Ok(filter)
    }

    /// Get the contact list from relays.
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn get_contact_list(&self, timeout: Duration) -> Result<Vec<Contact>, Error> {
        let mut contact_list: Vec<Contact> = Vec::new();
        let filter: Filter = self.get_contact_list_filter().await?;
        let events: Events = self.fetch_events(filter, timeout).await?;

        // Get first event (result of `fetch_events` is sorted DESC by timestamp)
        if let Some(event) = events.first_owned() {
            for tag in event.tags.into_iter() {
                if let Some(TagStandard::PublicKey {
                    public_key,
                    relay_url,
                    alias,
                    uppercase: false,
                }) = tag.to_standardized()
                {
                    contact_list.push(Contact {
                        public_key,
                        relay_url,
                        alias,
                    })
                }
            }
        }

        Ok(contact_list)
    }

    /// Get contact list public keys from relays.
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn get_contact_list_public_keys(
        &self,
        timeout: Duration,
    ) -> Result<Vec<PublicKey>, Error> {
        let mut pubkeys: Vec<PublicKey> = Vec::new();
        let filter: Filter = self.get_contact_list_filter().await?;
        let events: Events = self.fetch_events(filter, timeout).await?;

        for event in events.into_iter() {
            pubkeys.extend(event.tags.public_keys());
        }

        Ok(pubkeys)
    }

    /// Get contact list [`Metadata`] from relays.
    ///
    /// This method requires a [`NostrSigner`].
    pub async fn get_contact_list_metadata(
        &self,
        timeout: Duration,
    ) -> Result<HashMap<PublicKey, Metadata>, Error> {
        let public_keys = self.get_contact_list_public_keys(timeout).await?;
        let mut contacts: HashMap<PublicKey, Metadata> =
            public_keys.iter().map(|p| (*p, Metadata::new())).collect();

        let filter: Filter = Filter::new().authors(public_keys).kind(Kind::Metadata);
        let events: Events = self.fetch_events(filter, timeout).await?;
        for event in events.into_iter() {
            let metadata = Metadata::from_json(&event.content)?;
            if let Some(m) = contacts.get_mut(&event.pubkey) {
                *m = metadata
            };
        }

        Ok(contacts)
    }

    /// Send a private direct message
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the message will be sent to the NIP17 relays (automatically discovered).
    /// If gossip is not enabled will be sent to all relays with [`RelayServiceFlags::WRITE`] flag.
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::PrivateMsgRelaysNotFound`] if the receiver hasn't set the NIP17 list,
    /// meaning that is not ready to receive private messages.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn send_private_msg<S, I>(
        &self,
        receiver: PublicKey,
        message: S,
        rumor_extra_tags: I,
    ) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
        I: IntoIterator<Item = Tag>,
    {
        let signer = self.signer().await?;
        let event: Event =
            EventBuilder::private_msg(&signer, receiver, message, rumor_extra_tags).await?;

        // NOT gossip, send to all relays
        if !self.opts.gossip {
            return self.send_event(&event).await;
        }

        self.gossip_send_event(&event, true).await
    }

    /// Send a private direct message to specific relays
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn send_private_msg_to<I, S, U, IT>(
        &self,
        urls: I,
        receiver: PublicKey,
        message: S,
        rumor_extra_tags: IT,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        S: Into<String>,
        U: TryIntoUrl,
        IT: IntoIterator<Item = Tag>,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let signer = self.signer().await?;
        let event: Event =
            EventBuilder::private_msg(&signer, receiver, message, rumor_extra_tags).await?;
        self.send_event_to(urls, &event).await
    }

    /// Construct Gift Wrap and send to relays
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// Check [`Client::send_event`] to know how sending events works.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn gift_wrap<I>(
        &self,
        receiver: &PublicKey,
        rumor: UnsignedEvent,
        extra_tags: I,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = Tag>,
    {
        // Acquire signer
        let signer = self.signer().await?;

        // Build gift wrap
        let gift_wrap: Event =
            EventBuilder::gift_wrap(&signer, receiver, rumor, extra_tags).await?;

        // Send
        self.send_event(&gift_wrap).await
    }

    /// Construct Gift Wrap and send to specific relays
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn gift_wrap_to<I, U, IT>(
        &self,
        urls: I,
        receiver: &PublicKey,
        rumor: UnsignedEvent,
        extra_tags: IT,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        IT: IntoIterator<Item = Tag>,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        // Acquire signer
        let signer = self.signer().await?;

        // Build gift wrap
        let gift_wrap: Event =
            EventBuilder::gift_wrap(&signer, receiver, rumor, extra_tags).await?;

        // Send
        self.send_event_to(urls, &gift_wrap).await
    }

    /// Unwrap Gift Wrap event
    ///
    /// This method requires a [`NostrSigner`].
    ///
    /// Check [`UnwrappedGift::from_gift_wrap`] to learn more.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn unwrap_gift_wrap(&self, gift_wrap: &Event) -> Result<UnwrappedGift, Error> {
        let signer = self.signer().await?;
        Ok(UnwrappedGift::from_gift_wrap(&signer, gift_wrap).await?)
    }

    /// Handle notifications
    ///
    /// The closure function expects a `bool` as output: return `true` to exit from the notification loop.
    #[inline]
    pub async fn handle_notifications<F, Fut>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(RelayPoolNotification) -> Fut,
        Fut: Future<Output = Result<bool>>,
    {
        Ok(self.pool.handle_notifications(func).await?)
    }
}

// Gossip
impl Client {
    /// Check if there are outdated public keys and update them
    async fn check_and_update_gossip<I>(&self, public_keys: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let outdated_public_keys: HashSet<PublicKey> =
            self.gossip.check_outdated(public_keys).await;

        // No outdated public keys, immediately return.
        if outdated_public_keys.is_empty() {
            return Ok(());
        }

        // Compose filters
        let filter: Filter = Filter::default()
            .authors(outdated_public_keys.clone())
            .kinds([Kind::RelayList, Kind::InboxRelays]);

        // Query from database
        let stored_events: Events = self.database().query(filter.clone()).await?;

        // Get DISCOVERY and READ relays
        let urls: Vec<RelayUrl> = self
            .pool
            .__relay_urls_with_flag(
                RelayServiceFlags::DISCOVERY | RelayServiceFlags::READ,
                FlagCheck::Any,
            )
            .await;

        // Get events from discovery and read relays
        let events: Events = self
            .fetch_events_from(urls, filter, Duration::from_secs(10))
            .await?;

        // Update last check for these public keys
        self.gossip.update_last_check(outdated_public_keys).await;

        // Merge database and relays events
        let merged: Events = events.merge(stored_events);

        // Update gossip graph
        self.gossip.update(merged).await;

        Ok(())
    }

    /// Break down filters for gossip and discovery relays
    async fn break_down_filter(&self, filter: Filter) -> Result<HashMap<RelayUrl, Filter>, Error> {
        // Extract all public keys from filters
        let public_keys = filter.extract_public_keys();

        // Check and update outdated public keys
        self.check_and_update_gossip(public_keys).await?;

        // Broken-down filters
        let filters: HashMap<RelayUrl, Filter> = match self.gossip.break_down_filter(filter).await {
            BrokenDownFilters::Filters(filters) => filters,
            BrokenDownFilters::Orphan(filter) | BrokenDownFilters::Other(filter) => {
                // Get read relays
                let read_relays: Vec<RelayUrl> = self.pool.__read_relay_urls().await;

                let mut map = HashMap::with_capacity(read_relays.len());
                for url in read_relays.into_iter() {
                    map.insert(url, filter.clone());
                }
                map
            }
        };

        // Add gossip (outbox and inbox) relays
        for url in filters.keys() {
            if self.add_gossip_relay(url).await? {
                self.connect_relay(url).await?;
            }
        }

        // Check if filters are empty
        // TODO: this can't be empty, right?
        if filters.is_empty() {
            return Err(Error::GossipFiltersEmpty);
        }

        Ok(filters)
    }

    async fn gossip_send_event(
        &self,
        event: &Event,
        is_nip17: bool,
    ) -> Result<Output<EventId>, Error> {
        let is_gift_wrap: bool = event.kind == Kind::GiftWrap;

        // Get involved public keys and check what are up to date in the gossip graph and which ones require an update.
        if is_gift_wrap {
            // Get only p tags since the author of a gift wrap is randomized
            let public_keys = event.tags.public_keys().copied();
            self.check_and_update_gossip(public_keys).await?;
        } else {
            // Get all public keys involved in the event: author + p tags
            let public_keys = event
                .tags
                .public_keys()
                .copied()
                .chain(iter::once(event.pubkey));
            self.check_and_update_gossip(public_keys).await?;
        };

        // Check if NIP17 or NIP65
        let urls: HashSet<RelayUrl> = if is_nip17 && is_gift_wrap {
            // Get NIP17 relays
            // Get only for relays for p tags since gift wraps are signed with random key (random author)
            let relays = self
                .gossip
                .get_nip17_inbox_relays(event.tags.public_keys())
                .await;

            // Clients SHOULD publish kind 14 events to the 10050-listed relays.
            // If that is not found, that indicates the user is not ready to receive messages under this NIP and clients shouldn't try.
            //
            // <https://github.com/nostr-protocol/nips/blob/6e7a618e7f873bb91e743caacc3b09edab7796a0/17.md>
            if relays.is_empty() {
                return Err(Error::PrivateMsgRelaysNotFound);
            }

            // Add outbox and inbox relays
            for url in relays.iter() {
                if self.add_gossip_relay(url).await? {
                    self.connect_relay(url).await?;
                }
            }

            relays
        } else {
            // Get NIP65 relays
            let mut outbox = self.gossip.get_nip65_outbox_relays(&[event.pubkey]).await;
            let inbox = self
                .gossip
                .get_nip65_inbox_relays(event.tags.public_keys())
                .await;

            // Add outbox and inbox relays
            for url in outbox.iter().chain(inbox.iter()) {
                if self.add_gossip_relay(url).await? {
                    self.connect_relay(url).await?;
                }
            }

            // Get WRITE relays
            let write_relays: Vec<RelayUrl> = self.pool.__write_relay_urls().await;

            // Extend OUTBOX relays with WRITE ones
            outbox.extend(write_relays);

            // Extend outbox relays with inbox ones
            outbox.extend(inbox);

            // Return all relays
            outbox
        };

        // Send event
        Ok(self.pool.send_event_to(urls, event).await?)
    }

    async fn gossip_stream_events(
        &self,
        filter: Filter,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<ReceiverStream<Event>, Error> {
        let filters = self.break_down_filter(filter).await?;

        // Stream events
        let stream: ReceiverStream<Event> = self
            .pool
            .stream_events_targeted(filters, timeout, policy)
            .await?;

        Ok(stream)
    }

    async fn gossip_fetch_events(
        &self,
        filter: Filter,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<Events, Error> {
        let mut events: Events = Events::new(&filter);

        // Stream events
        let mut stream: ReceiverStream<Event> =
            self.gossip_stream_events(filter, timeout, policy).await?;

        while let Some(event) = stream.next().await {
            // To find out more about why the `force_insert` was used, search for EVENTS_FORCE_INSERT ine the code.
            events.force_insert(event);
        }

        Ok(events)
    }

    async fn gossip_subscribe(
        &self,
        id: SubscriptionId,
        filter: Filter,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error> {
        let filters = self.break_down_filter(filter).await?;
        Ok(self.pool.subscribe_targeted(id, filters, opts).await?)
    }

    async fn gossip_sync_negentropy(
        &self,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        // Break down filter
        let temp_filters = self.break_down_filter(filter).await?;

        let database = self.database();
        let mut filters: HashMap<RelayUrl, (Filter, Vec<_>)> =
            HashMap::with_capacity(temp_filters.len());

        // Iterate broken down filters and compose new filters for targeted reconciliation
        for (url, filter) in temp_filters.into_iter() {
            // Get items
            let items: Vec<(EventId, Timestamp)> =
                database.negentropy_items(filter.clone()).await?;

            filters.insert(url, (filter, items));
        }

        // Reconciliation
        Ok(self.pool.sync_targeted(filters, opts).await?)
    }
}
