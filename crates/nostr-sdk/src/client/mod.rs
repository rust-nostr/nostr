// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::iter;
use std::sync::Arc;
use std::time::Duration;

use atomic_destructor::StealthClone;
use nostr::prelude::*;
use nostr_database::DynNostrDatabase;
use nostr_relay_pool::prelude::*;
#[cfg(feature = "nip57")]
use nostr_zapper::{DynNostrZapper, IntoNostrZapper, ZapperError};
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};

pub mod builder;
mod handler;
pub mod options;
#[cfg(feature = "nip57")]
mod zapper;

pub use self::builder::ClientBuilder;
#[cfg(not(target_arch = "wasm32"))]
pub use self::options::{Connection, ConnectionTarget};
pub use self::options::{EventSource, Options};
#[cfg(feature = "nip57")]
pub use self::zapper::{ZapDetails, ZapEntity};
use crate::gossip::graph::GossipGraph;

/// [`Client`] error
#[derive(Debug, Error)]
pub enum Error {
    /// [`Relay`] error
    #[error(transparent)]
    Relay(#[from] nostr_relay_pool::relay::Error),
    /// [`RelayPool`] error
    #[error(transparent)]
    RelayPool(#[from] pool::Error),
    /// Database error
    #[error(transparent)]
    Database(#[from] DatabaseError),
    /// Signer error
    #[error(transparent)]
    Signer(#[from] SignerError),
    /// Zapper error
    #[cfg(feature = "nip57")]
    #[error(transparent)]
    Zapper(#[from] ZapperError),
    /// [`EventBuilder`] error
    #[error(transparent)]
    EventBuilder(#[from] event::builder::Error),
    /// Metadata error
    #[error(transparent)]
    Metadata(#[from] metadata::Error),
    /// Signer not configured
    #[error("signer not configured")]
    SignerNotConfigured,
    /// Zapper not configured
    #[cfg(feature = "nip57")]
    #[error("zapper not configured")]
    ZapperNotConfigured,
    /// NIP57 error
    #[cfg(feature = "nip57")]
    #[error(transparent)]
    NIP57(#[from] nip57::Error),
    /// LNURL Pay
    #[cfg(feature = "nip57")]
    #[error(transparent)]
    LnUrlPay(#[from] lnurl_pay::Error),
    /// NIP59
    #[cfg(feature = "nip59")]
    #[error(transparent)]
    NIP59(#[from] nip59::Error),
    /// Event not found
    #[error("event not found: {0}")]
    EventNotFound(EventId),
    /// Impossible to zap
    #[error("impossible to send zap: {0}")]
    ImpossibleToZap(String),
    /// Broken down filters for gossip are empty
    #[error("gossip broken down filters are empty")]
    GossipFiltersEmpty,
    /// Metadata not found
    #[error("metadata not found")]
    MetadataNotFound,
}

/// Nostr client
#[derive(Debug, Clone)]
pub struct Client {
    pool: RelayPool,
    signer: Arc<RwLock<Option<Arc<dyn NostrSigner>>>>,
    #[cfg(feature = "nip57")]
    zapper: Arc<RwLock<Option<Arc<DynNostrZapper>>>>,
    gossip_graph: GossipGraph,
    opts: Options,
}

impl Default for Client {
    #[inline]
    fn default() -> Self {
        Self::builder().build()
    }
}

impl StealthClone for Client {
    fn stealth_clone(&self) -> Self {
        Self {
            pool: self.pool.stealth_clone(),
            signer: self.signer.clone(),
            #[cfg(feature = "nip57")]
            zapper: self.zapper.clone(),
            gossip_graph: self.gossip_graph.clone(),
            opts: self.opts.clone(),
        }
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

    /// Construct client with signer and options
    #[deprecated(since = "0.37.0", note = "Use `Client::builder` instead")]
    pub fn with_opts<T>(signer: T, opts: Options) -> Self
    where
        T: IntoNostrSigner,
    {
        Self::builder().signer(signer).opts(opts).build()
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
    /// let opts = Options::default().connection_timeout(Some(Duration::from_secs(30)));
    /// let client: Client = Client::builder().signer(signer).opts(opts).build();
    /// ```
    #[inline]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    fn from_builder(builder: ClientBuilder) -> Self {
        let client = Self {
            pool: RelayPool::with_database(builder.opts.pool, builder.database),
            signer: Arc::new(RwLock::new(builder.signer)),
            #[cfg(feature = "nip57")]
            zapper: Arc::new(RwLock::new(builder.zapper)),
            gossip_graph: GossipGraph::new(),
            opts: builder.opts,
        };

        client.spawn_notification_handler();

        client
    }

    /// Update default difficulty for new [`Event`]
    #[inline]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.opts.update_difficulty(difficulty);
    }

    /// Update minimum POW difficulty for received events
    ///
    /// Events with a POW lower than the current value will be ignored to prevent resources exhaustion.
    #[inline]
    pub fn update_min_pow_difficulty(&self, difficulty: u8) {
        self.opts.update_min_pow_difficulty(difficulty);
    }

    /// Auto authenticate to relays (default: true)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub fn automatic_authentication(&self, enable: bool) {
        self.opts.update_automatic_authentication(enable);
    }

    /// Get current nostr signer
    ///
    /// Rise error if it not set.
    pub async fn signer(&self) -> Result<Arc<dyn NostrSigner>, Error> {
        let signer = self.signer.read().await;
        signer.clone().ok_or(Error::SignerNotConfigured)
    }

    /// Set nostr signer
    pub async fn set_signer<T>(&self, signer: T)
    where
        T: IntoNostrSigner,
    {
        let mut s = self.signer.write().await;
        *s = Some(signer.into_nostr_signer());
    }

    /// Unset nostr signer
    pub async fn unset_signer(&self) {
        let mut s = self.signer.write().await;
        *s = None;
    }

    /// Check if `zapper` is configured
    #[cfg(feature = "nip57")]
    pub async fn has_zapper(&self) -> bool {
        let zapper = self.zapper.read().await;
        zapper.is_some()
    }

    /// Get current nostr zapper
    ///
    /// Rise error if it not set.
    #[cfg(feature = "nip57")]
    pub async fn zapper(&self) -> Result<Arc<DynNostrZapper>, Error> {
        let zapper = self.zapper.read().await;
        zapper.clone().ok_or(Error::ZapperNotConfigured)
    }

    /// Set nostr zapper
    #[cfg(feature = "nip57")]
    pub async fn set_zapper<Z>(&self, zapper: Z)
    where
        Z: IntoNostrZapper,
    {
        let mut s = self.zapper.write().await;
        *s = Some(zapper.into_nostr_zapper());
    }

    /// Unset nostr zapper
    #[cfg(feature = "nip57")]
    pub async fn unset_zapper(&self) {
        let mut s = self.zapper.write().await;
        *s = None;
    }

    /// Get [`RelayPool`]
    #[inline]
    pub fn pool(&self) -> &RelayPool {
        &self.pool
    }

    /// Get database
    #[inline]
    pub fn database(&self) -> &Arc<DynNostrDatabase> {
        self.pool.database()
    }

    /// Get filtering
    #[inline]
    pub fn filtering(&self) -> &RelayFiltering {
        self.pool.filtering()
    }

    /// Reset client
    ///
    /// This method reset the client to simplify the switch to another account.
    ///
    /// This method will:
    /// * unsubscribe from all subscriptions
    /// * disconnect and force remove all relays
    /// * unset the signer
    /// * unset the zapper
    /// * clear the [`RelayFiltering`]
    ///
    /// This method will NOT:
    /// * reset [`Options`]
    /// * remove the database
    /// * clear the gossip graph
    pub async fn reset(&self) -> Result<(), Error> {
        self.unsubscribe_all().await;
        self.force_remove_all_relays().await?;
        self.unset_signer().await;
        #[cfg(feature = "nip57")]
        self.unset_zapper().await;
        self.filtering().clear().await;
        Ok(())
    }

    /// Completely shutdown client
    #[inline]
    pub async fn shutdown(&self) -> Result<(), Error> {
        Ok(self.pool.shutdown().await?)
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
    pub async fn relays(&self) -> HashMap<Url, Relay> {
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

    async fn compose_relay_opts(&self, _url: &Url) -> RelayOptions {
        let opts: RelayOptions = RelayOptions::new();

        // Set connection mode
        #[cfg(not(target_arch = "wasm32"))]
        let opts: RelayOptions = match &self.opts.connection.mode {
            ConnectionMode::Direct => opts,
            ConnectionMode::Proxy(..) => match self.opts.connection.target {
                ConnectionTarget::All => opts.connection_mode(self.opts.connection.mode.clone()),
                ConnectionTarget::Onion => {
                    let domain: &str = _url.domain().unwrap_or_default();

                    if domain.ends_with(".onion") {
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
                    let domain: &str = _url.domain().unwrap_or_default();

                    if domain.ends_with(".onion") {
                        opts.connection_mode(self.opts.connection.mode.clone())
                    } else {
                        opts
                    }
                }
            },
        };

        // Set min POW difficulty and limits
        opts.pow(self.opts.get_min_pow_difficulty())
            .limits(self.opts.relay_limits.clone())
            .max_avg_latency(self.opts.max_avg_latency)
    }

    /// If return `false` means that already existed
    async fn get_or_add_relay_with_flag<U>(
        &self,
        url: U,
        inherit_pool_subscriptions: bool,
        flag: RelayServiceFlags,
    ) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        // Convert into url
        let url: Url = url.try_into_url().map_err(pool::Error::from)?;

        // Compose relay options
        let opts: RelayOptions = self.compose_relay_opts(&url).await;

        // Set flag
        let opts: RelayOptions = opts.flags(flag);

        // Add relay with opts or edit current one
        match self
            .pool
            .get_or_add_relay::<&Url>(&url, inherit_pool_subscriptions, opts)
            .await?
        {
            Some(relay) => {
                relay.flags().add(flag);
                Ok(false)
            }
            None => {
                // Connect if `autoconnect` is enabled
                if self.opts.autoconnect {
                    self.connect_relay::<Url>(url).await?;
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
        self.get_or_add_relay_with_flag(url, true, RelayServiceFlags::default())
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
        self.get_or_add_relay_with_flag(
            url,
            false,
            RelayServiceFlags::PING | RelayServiceFlags::DISCOVERY,
        )
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
        self.get_or_add_relay_with_flag(
            url,
            true,
            RelayServiceFlags::PING | RelayServiceFlags::READ,
        )
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
        self.get_or_add_relay_with_flag(
            url,
            false,
            RelayServiceFlags::PING | RelayServiceFlags::WRITE,
        )
        .await
    }

    #[inline]
    async fn add_inbox_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.get_or_add_relay_with_flag(
            url,
            false,
            RelayServiceFlags::PING | RelayServiceFlags::INBOX,
        )
        .await
    }

    #[inline]
    async fn add_outbox_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.get_or_add_relay_with_flag(
            url,
            false,
            RelayServiceFlags::PING | RelayServiceFlags::OUTBOX,
        )
        .await
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has [`RelayServiceFlags::INBOX`] or [`RelayServiceFlags::OUTBOX`] flags, it will not be removed from the pool and its
    /// flags will be updated (remove [`RelayServiceFlags::READ`], [`RelayServiceFlags::WRITE`] and [`RelayServiceFlags::DISCOVERY`] flags).
    ///
    /// To fore remove it use [`Client::force_remove_relay`].
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
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        Ok(self.pool.remove_all_relays().await?)
    }

    /// Disconnect and force remove all relays
    #[inline]
    pub async fn force_remove_all_relays(&self) -> Result<(), Error> {
        Ok(self.pool.force_remove_all_relays().await?)
    }

    /// Connect to a previously added relay
    #[inline]
    pub async fn connect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self
            .pool
            .connect_relay(url, self.opts.connection_timeout)
            .await?)
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
    #[inline]
    pub async fn connect(&self) {
        self.pool.connect(self.opts.connection_timeout).await;
    }

    /// Connect to all added relays
    ///
    /// Try to connect to the relays and wait for them to be connected at most for the specified `timeout`.
    /// The code continues if the `timeout` is reached or if all relays connect.
    #[inline]
    pub async fn connect_with_timeout(&self, timeout: Duration) {
        self.pool.connect(Some(timeout)).await
    }

    /// Disconnect from all relays
    #[inline]
    pub async fn disconnect(&self) -> Result<(), Error> {
        Ok(self.pool.disconnect().await?)
    }

    /// Get pool subscriptions
    #[inline]
    pub async fn subscriptions(&self) -> HashMap<SubscriptionId, Vec<Filter>> {
        self.pool.subscriptions().await
    }

    /// Get pool subscription
    #[inline]
    pub async fn subscription(&self, id: &SubscriptionId) -> Option<Vec<Filter>> {
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
    /// let output = client.subscribe(vec![subscription], None).await?;
    /// println!("Subscription ID: {}", output.val);
    ///
    /// // Auto-closing subscription
    /// let id = SubscriptionId::generate();
    /// let subscription = Filter::new().kind(Kind::TextNote).limit(10);
    /// let opts = SubscribeAutoCloseOptions::default().filter(FilterOptions::ExitOnEOSE);
    /// let output = client.subscribe(vec![subscription], Some(opts)).await?;
    /// println!("Subscription ID: {} [auto-closing]", output.val);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(
        &self,
        filters: Vec<Filter>,
        opts: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<SubscriptionId>, Error> {
        let id: SubscriptionId = SubscriptionId::generate();
        let output: Output<()> = self.subscribe_with_id(id.clone(), filters, opts).await?;
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
        filters: Vec<Filter>,
        opts: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<()>, Error> {
        let opts: SubscribeOptions = SubscribeOptions::default().close_on(opts);

        if self.opts.gossip {
            self.gossip_subscribe(id, filters, opts).await
        } else {
            Ok(self.pool.subscribe_with_id(id, filters, opts).await?)
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
        filters: Vec<Filter>,
        opts: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<SubscriptionId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let opts: SubscribeOptions = SubscribeOptions::default().close_on(opts);
        Ok(self.pool.subscribe_to(urls, filters, opts).await?)
    }

    /// Subscribe to filters with custom [SubscriptionId] to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeAutoCloseOptions].
    #[inline]
    pub async fn subscribe_with_id_to<I, U>(
        &self,
        urls: I,
        id: SubscriptionId,
        filters: Vec<Filter>,
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
            .subscribe_with_id_to(urls, id, filters, opts)
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
        I: IntoIterator<Item = (U, Vec<Filter>)>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.subscribe_targeted(id, targets, opts).await?)
    }

    /// Unsubscribe
    #[inline]
    pub async fn unsubscribe(&self, id: SubscriptionId) {
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
    ///     .fetch_events(vec![subscription], Some(Duration::from_secs(10)))
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn fetch_events(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Events, Error> {
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);

        if self.opts.gossip {
            return self.gossip_fetch_events(filters, timeout).await;
        }

        Ok(self
            .pool
            .fetch_events(filters, timeout, FilterOptions::ExitOnEOSE)
            .await?)
    }

    /// Get events of filters
    #[deprecated(since = "0.36.0", note = "Use `fetch_events` instead")]
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        _source: EventSource,
    ) -> Result<Vec<Event>, Error> {
        Ok(self.fetch_events(filters, None).await?.to_vec())
    }

    /// Fetch events from specific relays
    #[inline]
    pub async fn fetch_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Events, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
        Ok(self
            .pool
            .fetch_events_from(urls, filters, timeout, FilterOptions::ExitOnEOSE)
            .await?)
    }

    /// Fetch events of filters from specific relays
    #[deprecated(since = "0.36.0", note = "Use `fetch_events_from` instead")]
    pub async fn get_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self
            .fetch_events_from(urls, filters, timeout)
            .await?
            .to_vec())
    }

    /// Get events both from database and relays
    ///
    ///
    /// This method will be deprecated in the future!
    /// This is a temporary solution for who still want to query events both from database and relays and merge the result.
    /// The optimal solution is to execute a [`Client::sync`] to reconcile missing events, [`Client::subscribe`] to get all
    /// new future events, [`NostrDatabase::query`] to query stored events and [`Client::handle_notifications`] to listen-for/handle new events (i.e. to know when update the UI).
    /// This will allow very fast queries, low bandwidth usage (depending on how many events the client have to reconcile) and a lower load on the relays.
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be requested also to
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// You can obtain the same result with:
    /// ```rust,no_run
    /// # use std::time::Duration;
    /// # use nostr_sdk::prelude::*;
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// # let client = Client::default();
    /// # let filters = vec![Filter::new().limit(1)];
    /// // Query database
    /// let stored_events: Events = client.database().query(filters.clone()).await?;
    ///
    /// // Query relays
    /// let fetched_events: Events = client
    ///     .fetch_events(filters, Some(Duration::from_secs(10)))
    ///     .await?;
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
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Events, Error> {
        // Query database
        let stored_events: Events = self.database().query(filters.clone()).await?;

        // Query relays
        let fetched_events: Events = self.fetch_events(filters, timeout).await?;

        // Merge result
        Ok(stored_events.merge(fetched_events))
    }

    /// Stream events
    #[deprecated(since = "0.36.0", note = "Use `stream_events` instead")]
    pub async fn stream_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<ReceiverStream<Event>, Error> {
        self.stream_events(filters, timeout).await
    }

    /// Stream events from relays
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be streamed also from
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    pub async fn stream_events(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<ReceiverStream<Event>, Error> {
        // Get timeout
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);

        // Check if gossip is enabled
        if self.opts.gossip {
            self.gossip_stream_events(filters, timeout).await
        } else {
            Ok(self
                .pool
                .stream_events(filters, timeout, FilterOptions::ExitOnEOSE)
                .await?)
        }
    }

    /// Stream events from specific relays
    #[inline]
    pub async fn stream_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
        Ok(self
            .pool
            .stream_events_from(urls, filters, timeout, FilterOptions::ExitOnEOSE)
            .await?)
    }

    /// Targeted streaming events
    ///
    /// Stream events from specific relays with specific filters
    pub async fn stream_events_targeted<I, U>(
        &self,
        source: I,
        timeout: Option<Duration>,
    ) -> Result<ReceiverStream<Event>, Error>
    where
        I: IntoIterator<Item = (U, Vec<Filter>)>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
        Ok(self
            .pool
            .stream_events_targeted(source, timeout, FilterOptions::ExitOnEOSE)
            .await?)
    }

    /// Send client message to a **specific relays**
    #[inline]
    pub async fn send_msg_to<I, U>(&self, urls: I, msg: ClientMessage) -> Result<Output<()>, Error>
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
        msgs: Vec<ClientMessage>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.batch_msg_to(urls, msgs).await?)
    }

    /// Send event
    ///
    /// Send [`Event`] to all relays with [`RelayServiceFlags::WRITE`] flag.
    /// If `gossip` is enabled (see [`Options::gossip`]) the event will be sent also to NIP65 relays (automatically discovered).
    #[inline]
    pub async fn send_event(&self, event: Event) -> Result<Output<EventId>, Error> {
        // NOT gossip, send event to all relays
        if !self.opts.gossip {
            return Ok(self.pool.send_event(event).await?);
        }

        // ########## Gossip ##########

        // Get all public keys involved in the event
        let public_keys = event
            .tags
            .public_keys()
            .copied()
            .chain(iter::once(event.pubkey));

        // Check what are up-to-date in the gossip graph and which ones require an update
        let outdated_public_keys = self.gossip_graph.check_outdated(public_keys).await;
        self.update_outdated_gossip_graph(outdated_public_keys)
            .await?;

        // Get relays
        let mut outbox = self.gossip_graph.get_outbox_relays(&[event.pubkey]).await;
        let inbox = self
            .gossip_graph
            .get_inbox_relays(event.tags.public_keys())
            .await;

        // Add outbox relays
        for url in outbox.iter() {
            if self.add_outbox_relay(url).await? {
                self.connect_relay(url).await?;
            }
        }

        // Add inbox relays
        for url in inbox.iter() {
            if self.add_inbox_relay(url).await? {
                self.connect_relay(url).await?;
            }
        }

        // Get WRITE relays
        // TODO: avoid clone of both url and relay
        let write_relays = self
            .pool
            .relays_with_flag(RelayServiceFlags::WRITE, FlagCheck::All)
            .await
            .into_keys();

        // Extend OUTBOX relays with WRITE ones
        outbox.extend(write_relays);

        // Union of OUTBOX (and WRITE) with INBOX relays
        let urls = outbox.union(&inbox);

        // Send event
        Ok(self.pool.send_event_to(urls, event).await?)
    }

    /// Send multiple events at once to all relays with [`RelayServiceFlags::WRITE`] flag.
    #[inline]
    pub async fn batch_event(&self, events: Vec<Event>) -> Result<Output<()>, Error> {
        Ok(self.pool.batch_event(events).await?)
    }

    /// Send event to specific relays.
    #[inline]
    pub async fn send_event_to<I, U>(&self, urls: I, event: Event) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.send_event_to(urls, event).await?)
    }

    /// Send multiple events at once to specific relays
    #[inline]
    pub async fn batch_event_to<I, U>(
        &self,
        urls: I,
        events: Vec<Event>,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.batch_event_to(urls, events).await?)
    }

    /// Signs the [`EventBuilder`] into an [`Event`] using the [`NostrSigner`]
    pub async fn sign_event_builder(&self, builder: EventBuilder) -> Result<Event, Error> {
        let signer = self.signer().await?;

        let public_key: PublicKey = signer.get_public_key().await?;
        let difficulty: u8 = self.opts.get_difficulty();
        let unsigned: UnsignedEvent = builder.pow(difficulty).build(public_key);

        Ok(signer.sign_event(unsigned).await?)
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to relays (check [`Client::send_event`] from more details).
    ///
    /// Rise an error if the [`NostrSigner`] is not set.
    #[inline]
    pub async fn send_event_builder(
        &self,
        builder: EventBuilder,
    ) -> Result<Output<EventId>, Error> {
        let event: Event = self.sign_event_builder(builder).await?;
        self.send_event(event).await
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to specific relays.
    ///
    /// Rise an error if the [`NostrSigner`] is not set.
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
        self.send_event_to(urls, event).await
    }

    /// Fetch the newest public key metadata from relays.
    ///
    /// If you only want to consult stored data,
    /// consider `client.database().profile(PUBKEY)`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub async fn fetch_metadata(
        &self,
        public_key: PublicKey,
        timeout: Option<Duration>,
    ) -> Result<Metadata, Error> {
        let filter: Filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        // TODO: add fetch_event and use that
        let events: Events = self.fetch_events(vec![filter], timeout).await?;
        match events.first() {
            Some(event) => Ok(Metadata::try_from(event)?),
            None => Err(Error::MetadataNotFound),
        }
    }

    /// Update metadata
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

    /// Set relay list (NIP65)
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[inline]
    pub async fn set_relay_list<I>(&self, relays: I) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = (Url, Option<RelayMetadata>)>,
    {
        let builder = EventBuilder::relay_list(relays);
        self.send_event_builder(builder).await
    }

    /// Publish text note
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub async fn publish_text_note<S, I>(
        &self,
        content: S,
        tags: I,
    ) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
        I: IntoIterator<Item = Tag>,
    {
        let builder = EventBuilder::text_note(content, tags);
        self.send_event_builder(builder).await
    }

    /// Set contact list
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    #[inline]
    pub async fn set_contact_list<I>(&self, list: I) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = Contact>,
    {
        let builder = EventBuilder::contact_list(list);
        self.send_event_builder(builder).await
    }

    async fn get_contact_list_filters(&self) -> Result<Vec<Filter>, Error> {
        let signer = self.signer().await?;
        let public_key = signer.get_public_key().await?;
        let filter: Filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        Ok(vec![filter])
    }

    /// Get contact list from relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn get_contact_list(&self, timeout: Option<Duration>) -> Result<Vec<Contact>, Error> {
        let mut contact_list: Vec<Contact> = Vec::new();
        let filters: Vec<Filter> = self.get_contact_list_filters().await?;
        let events: Events = self.fetch_events(filters, timeout).await?;

        // Get first event (result of `fetch_events` is sorted DESC by timestamp)
        if let Some(event) = events.into_iter().next() {
            for tag in event.tags.into_iter() {
                if let Some(TagStandard::PublicKey {
                    public_key,
                    relay_url,
                    alias,
                    uppercase: false,
                }) = tag.to_standardized()
                {
                    contact_list.push(Contact::new(public_key, relay_url, alias))
                }
            }
        }

        Ok(contact_list)
    }

    /// Get contact list public keys from relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn get_contact_list_public_keys(
        &self,
        timeout: Option<Duration>,
    ) -> Result<Vec<PublicKey>, Error> {
        let mut pubkeys: Vec<PublicKey> = Vec::new();
        let filters: Vec<Filter> = self.get_contact_list_filters().await?;
        let events: Events = self.fetch_events(filters, timeout).await?;

        for event in events.into_iter() {
            pubkeys.extend(event.tags.public_keys());
        }

        Ok(pubkeys)
    }

    /// Get contact list [`Metadata`] from relays.
    pub async fn get_contact_list_metadata(
        &self,
        timeout: Option<Duration>,
    ) -> Result<HashMap<PublicKey, Metadata>, Error> {
        let public_keys = self.get_contact_list_public_keys(timeout).await?;
        let mut contacts: HashMap<PublicKey, Metadata> =
            public_keys.iter().map(|p| (*p, Metadata::new())).collect();

        let chunk_size: usize = self.opts.req_filters_chunk_size as usize;
        for chunk in public_keys.chunks(chunk_size) {
            let mut filters: Vec<Filter> = Vec::new();
            for public_key in chunk.iter() {
                filters.push(
                    Filter::new()
                        .author(*public_key)
                        .kind(Kind::Metadata)
                        .limit(1),
                );
            }
            let events: Events = self.fetch_events(filters, timeout).await?;
            for event in events.into_iter() {
                let metadata = Metadata::from_json(&event.content)?;
                if let Some(m) = contacts.get_mut(&event.pubkey) {
                    *m = metadata
                };
            }
        }

        Ok(contacts)
    }

    /// Send private direct message to all relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn send_private_msg<S>(
        &self,
        receiver: PublicKey,
        message: S,
        reply_to: Option<EventId>,
    ) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
    {
        let rumor: EventBuilder = EventBuilder::private_msg_rumor(receiver, message, reply_to);
        self.gift_wrap(&receiver, rumor, None).await
    }

    /// Send private direct message to specific relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/17.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn send_private_msg_to<I, S, U>(
        &self,
        urls: I,
        receiver: PublicKey,
        message: S,
        reply_to: Option<EventId>,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        S: Into<String>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let rumor: EventBuilder = EventBuilder::private_msg_rumor(receiver, message, reply_to);
        self.gift_wrap_to(urls, &receiver, rumor, None).await
    }

    /// Repost
    #[inline]
    pub async fn repost(
        &self,
        event: &Event,
        relay_url: Option<UncheckedUrl>,
    ) -> Result<Output<EventId>, Error> {
        let builder = EventBuilder::repost(event, relay_url);
        self.send_event_builder(builder).await
    }

    /// Delete event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    #[inline]
    pub async fn delete_event<T>(&self, id: T) -> Result<Output<EventId>, Error>
    where
        T: Into<EventIdOrCoordinate>,
    {
        let builder = EventBuilder::delete([id]);
        self.send_event_builder(builder).await
    }

    /// Like event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    #[inline]
    pub async fn like(&self, event: &Event) -> Result<Output<EventId>, Error> {
        self.reaction(event, "+").await
    }

    /// Disike event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    #[inline]
    pub async fn dislike(&self, event: &Event) -> Result<Output<EventId>, Error> {
        self.reaction(event, "-").await
    }

    /// React to an [`Event`]
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/25.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::str::FromStr;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let keys = Keys::generate();
    /// #   let client = Client::new(keys);
    /// let event =
    ///     Event::from_json(r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#)
    ///         .unwrap();
    ///
    /// client.reaction(&event, "🐻").await.unwrap();
    /// # }
    /// ```
    #[inline]
    pub async fn reaction<S>(&self, event: &Event, reaction: S) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::reaction(event, reaction);
        self.send_event_builder(builder).await
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub async fn new_channel(&self, metadata: &Metadata) -> Result<Output<EventId>, Error> {
        let builder = EventBuilder::channel(metadata);
        self.send_event_builder(builder).await
    }

    /// Update channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub async fn set_channel_metadata(
        &self,
        channel_id: EventId,
        relay_url: Option<Url>,
        metadata: &Metadata,
    ) -> Result<Output<EventId>, Error> {
        let builder = EventBuilder::channel_metadata(channel_id, relay_url, metadata);
        self.send_event_builder(builder).await
    }

    /// Send message to channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub async fn send_channel_msg<S>(
        &self,
        channel_id: EventId,
        relay_url: Url,
        msg: S,
    ) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::channel_msg(channel_id, relay_url, msg);
        self.send_event_builder(builder).await
    }

    /// Hide channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub async fn hide_channel_msg<S>(
        &self,
        message_id: EventId,
        reason: Option<S>,
    ) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::hide_channel_msg(message_id, reason);
        self.send_event_builder(builder).await
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    #[inline]
    pub async fn mute_channel_user<S>(
        &self,
        pubkey: PublicKey,
        reason: Option<S>,
    ) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::mute_channel_user(pubkey, reason);
        self.send_event_builder(builder).await
    }

    /// Create an auth event
    ///
    /// Send the event ONLY to the target relay.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    #[inline]
    pub async fn auth<S>(&self, challenge: S, relay: Url) -> Result<(), Error>
    where
        S: Into<String>,
    {
        // Construct event
        let builder: EventBuilder = EventBuilder::auth(challenge, relay.clone());
        let event: Event = self.sign_event_builder(builder).await?;

        // Get relay
        let relay: Relay = self.relay(relay).await?;

        // Send AUTH message
        relay.auth(event).await?;

        Ok(())
    }

    /// Create zap receipt event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[inline]
    #[cfg(feature = "nip57")]
    pub async fn zap_receipt<S>(
        &self,
        bolt11: S,
        preimage: Option<S>,
        zap_request: &Event,
    ) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::zap_receipt(bolt11, preimage, zap_request);
        self.send_event_builder(builder).await
    }

    /// Send a Zap!
    #[inline]
    #[cfg(feature = "nip57")]
    pub async fn zap<T>(
        &self,
        to: T,
        satoshi: u64,
        details: Option<ZapDetails>,
    ) -> Result<(), Error>
    where
        T: Into<ZapEntity>,
    {
        self.internal_zap(to, satoshi, details).await
    }

    /// Construct Gift Wrap and send to relays
    ///
    /// Check [`Client::send_event`] to know how sending events works.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn gift_wrap(
        &self,
        receiver: &PublicKey,
        rumor: EventBuilder,
        expiration: Option<Timestamp>,
    ) -> Result<Output<EventId>, Error> {
        // Acquire signer
        let signer = self.signer().await?;

        // Compose rumor
        let public_key: PublicKey = signer.get_public_key().await?;
        let rumor: UnsignedEvent = rumor.build(public_key);

        // Build gift wrap
        let gift_wrap: Event =
            EventBuilder::gift_wrap(&signer, receiver, rumor, expiration).await?;

        // Send
        self.send_event(gift_wrap).await
    }

    /// Construct Gift Wrap and send to specific relays
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn gift_wrap_to<I, U>(
        &self,
        urls: I,
        receiver: &PublicKey,
        rumor: EventBuilder,
        expiration: Option<Timestamp>,
    ) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        // Acquire signer
        let signer = self.signer().await?;

        // Compose rumor
        let public_key: PublicKey = signer.get_public_key().await?;
        let rumor: UnsignedEvent = rumor.build(public_key);

        // Build gift wrap
        let gift_wrap: Event =
            EventBuilder::gift_wrap(&signer, receiver, rumor, expiration).await?;

        // Send
        self.send_event_to(urls, gift_wrap).await
    }

    /// Unwrap Gift Wrap event
    ///
    /// Internally verify the `seal` event.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[inline]
    #[cfg(feature = "nip59")]
    pub async fn unwrap_gift_wrap(&self, gift_wrap: &Event) -> Result<UnwrappedGift, Error> {
        let signer = self.signer().await?;
        Ok(UnwrappedGift::from_gift_wrap(&signer, gift_wrap).await?)
    }

    /// File metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/94.md>
    #[inline]
    pub async fn file_metadata<S>(
        &self,
        description: S,
        metadata: FileMetadata,
    ) -> Result<Output<EventId>, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::file_metadata(description, metadata);
        self.send_event_builder(builder).await
    }

    /// Sync events with relays (negentropy reconciliation)
    ///
    /// If `gossip` is enabled (see [`Options::gossip`]) the events will be reconciled also from
    /// NIP65 relays (automatically discovered) of public keys included in filters (if any).
    ///
    /// <https://github.com/hoytech/negentropy>
    #[deprecated(since = "0.36.0", note = "Use `sync` instead")]
    pub async fn reconcile(
        &self,
        filter: Filter,
        opts: SyncOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        self.sync(filter, &opts).await
    }

    /// Sync events with specific relays (negentropy reconciliation)
    ///
    /// <https://github.com/hoytech/negentropy>
    #[deprecated(since = "0.36.0", note = "Use `sync_with` instead")]
    pub async fn reconcile_with<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: SyncOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.sync_with(urls, filter, &opts).await
    }

    /// Handle notifications
    ///
    /// The closure function expect a `bool` as result: `true` means "exit from the notifications loop".
    #[inline]
    pub async fn handle_notifications<F, Fut>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(RelayPoolNotification) -> Fut,
        Fut: Future<Output = Result<bool>>,
    {
        Ok(self.pool.handle_notifications(func).await?)
    }
}

impl Client {
    async fn update_outdated_gossip_graph(
        &self,
        outdated_public_keys: HashSet<PublicKey>,
    ) -> Result<(), Error> {
        if !outdated_public_keys.is_empty() {
            // Compose filters
            let filter: Filter = Filter::default()
                .authors(outdated_public_keys)
                .kind(Kind::RelayList);

            // Query from database
            let database = self.database();
            let stored_events: Events = database.query(vec![filter.clone()]).await?;

            // Get DISCOVERY and READ relays
            // TODO: avoid clone of both url and relay
            let relays = self
                .pool
                .relays_with_flag(
                    RelayServiceFlags::DISCOVERY | RelayServiceFlags::READ,
                    FlagCheck::Any,
                )
                .await
                .into_keys();

            // Get events from discovery and read relays
            let events: Events = self
                .fetch_events_from(relays, vec![filter], Some(Duration::from_secs(10)))
                .await?;

            // Merge database and relays events
            let merged: Events = events.merge(stored_events);

            // Update gossip graph
            self.gossip_graph.update(merged).await;
        }

        Ok(())
    }

    /// Break down filters for gossip and discovery relays
    async fn break_down_filters(
        &self,
        filters: Vec<Filter>,
    ) -> Result<HashMap<Url, Vec<Filter>>, Error> {
        // Extract all public keys from filters
        let public_keys = filters.iter().flat_map(|f| f.extract_public_keys());

        // Check outdated ones
        let outdated_public_keys = self.gossip_graph.check_outdated(public_keys).await;

        // Update outdated public keys
        self.update_outdated_gossip_graph(outdated_public_keys)
            .await?;

        // Broken down filters
        let mut broken_down = self.gossip_graph.break_down_filters(filters).await;

        // Get read relays
        let read_relays = self
            .pool
            .relays_with_flag(RelayServiceFlags::READ, FlagCheck::All)
            .await;

        // Extend filters with read relays and "other" filters (the filters that aren't linked to public keys)
        if let Some(other) = broken_down.other {
            for url in read_relays.into_keys() {
                broken_down
                    .filters
                    .entry(url)
                    .and_modify(|f| {
                        f.extend(other.clone());
                    })
                    .or_default()
                    .extend(other.clone())
            }
        }

        // Add outbox relays
        for url in broken_down.outbox_urls.into_iter() {
            if self.add_outbox_relay(&url).await? {
                self.connect_relay(url).await?;
            }
        }

        // Add inbox relays
        for url in broken_down.inbox_urls.into_iter() {
            if self.add_inbox_relay(&url).await? {
                self.connect_relay(url).await?;
            }
        }

        // Check if filters aren't empty
        if broken_down.filters.is_empty() {
            return Err(Error::GossipFiltersEmpty);
        }

        Ok(broken_down.filters)
    }

    async fn gossip_stream_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<ReceiverStream<Event>, Error> {
        let filters = self.break_down_filters(filters).await?;

        // Stream events
        let stream: ReceiverStream<Event> = self
            .pool
            .stream_events_targeted(filters, timeout, FilterOptions::ExitOnEOSE)
            .await?;

        Ok(stream)
    }

    async fn gossip_fetch_events(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<Events, Error> {
        let mut events: Events = Events::new(&filters);

        // Stream events
        let mut stream: ReceiverStream<Event> = self.gossip_stream_events(filters, timeout).await?;

        while let Some(event) = stream.next().await {
            events.insert(event);
        }

        Ok(events)
    }

    async fn gossip_subscribe(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: SubscribeOptions,
    ) -> Result<Output<()>, Error> {
        let filters = self.break_down_filters(filters).await?;
        Ok(self.pool.subscribe_targeted(id, filters, opts).await?)
    }

    async fn gossip_sync_negentropy(
        &self,
        filter: Filter,
        opts: &SyncOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        // Break down filter
        let temp_filters = self.break_down_filters(vec![filter]).await?;

        let database = self.database();
        let mut filters = HashMap::with_capacity(temp_filters.len());

        // Iterate broken down filters and compose new filters for targeted reconciliation
        for (url, value) in temp_filters.into_iter() {
            let mut map = HashMap::with_capacity(value.len());

            // Iterate per-url filters and get items
            for filter in value.into_iter() {
                // Get items
                let items: Vec<(EventId, Timestamp)> =
                    database.negentropy_items(filter.clone()).await?;

                // Add filter and items to map
                map.insert(filter, items);
            }

            filters.insert(url, map);
        }

        // Reconciliation
        Ok(self.pool.sync_targeted(filters, opts).await?)
    }
}
