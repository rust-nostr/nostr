// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client

use std::collections::btree_set::IntoIter;
use std::collections::{BTreeSet, HashMap};
use std::future::Future;
use std::iter::Rev;
use std::sync::Arc;
use std::time::Duration;

use atomic_destructor::StealthClone;
use nostr::prelude::*;
use nostr_database::DynNostrDatabase;
use nostr_relay_pool::prelude::*;
use nostr_signer::prelude::*;
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
    Signer(#[from] nostr_signer::Error),
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
    /// Event not found
    #[error("event not found: {0}")]
    EventNotFound(EventId),
    /// Impossible to zap
    #[error("impossible to send zap: {0}")]
    ImpossibleToZap(String),
    /// Metadata not found
    #[error("metadata not found")]
    MetadataNotFound,
}

/// Nostr client
#[derive(Debug, Clone)]
pub struct Client {
    pool: RelayPool,
    signer: Arc<RwLock<Option<NostrSigner>>>,
    #[cfg(feature = "nip57")]
    zapper: Arc<RwLock<Option<Arc<DynNostrZapper>>>>,
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
            opts: self.opts.clone(),
        }
    }
}

impl Client {
    /// Create a new [`Client`] with signer
    ///
    /// To create a [`Client`] without any signer use `Client::default()`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// let my_keys = Keys::generate();
    /// let client = Client::new(&my_keys);
    /// ```
    #[inline]
    pub fn new<S>(signer: S) -> Self
    where
        S: Into<NostrSigner>,
    {
        Self::builder().signer(signer).build()
    }

    /// Create a new [`Client`] with [`Options`]
    ///
    /// To create a [`Client`] with custom [`Options`] and without any signer use `Client::builder().opts(opts).build()`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// let my_keys = Keys::generate();
    /// let opts = Options::new().wait_for_send(true);
    /// let client = Client::with_opts(&my_keys, opts);
    /// ```
    #[inline]
    pub fn with_opts<S>(signer: S, opts: Options) -> Self
    where
        S: Into<NostrSigner>,
    {
        Self::builder().signer(signer).opts(opts).build()
    }

    /// Construct [ClientBuilder]
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

    /// Compose [`Client`] from [`ClientBuilder`]
    pub fn from_builder(builder: ClientBuilder) -> Self {
        let client = Self {
            pool: RelayPool::with_database(builder.opts.pool, builder.database),
            signer: Arc::new(RwLock::new(builder.signer)),
            #[cfg(feature = "nip57")]
            zapper: Arc::new(RwLock::new(builder.zapper)),
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
    pub async fn signer(&self) -> Result<NostrSigner, Error> {
        let signer = self.signer.read().await;
        signer.clone().ok_or(Error::SignerNotConfigured)
    }

    /// Set nostr signer
    pub async fn set_signer(&self, signer: Option<NostrSigner>) {
        let mut s = self.signer.write().await;
        *s = signer;
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
    pub fn pool(&self) -> RelayPool {
        self.pool.clone()
    }

    /// Get database
    #[inline]
    pub fn database(&self) -> Arc<DynNostrDatabase> {
        self.pool.database()
    }

    /// Get blacklist
    #[inline]
    pub fn blacklist(&self) -> RelayBlacklist {
        self.pool.blacklist()
    }

    /// Mute [EventId]s
    ///
    /// Add [EventId]s to blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    pub async fn mute_ids<I>(&self, ids: I)
    where
        I: IntoIterator<Item = EventId>,
    {
        let blacklist: RelayBlacklist = self.blacklist();
        blacklist.add_ids(ids).await;

        // TODO: create/update mute list event?
    }

    /// Unmute [EventId]s
    ///
    /// Remove [EventId]s from blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    pub async fn unmute_ids<'a, I>(&self, ids: I)
    where
        I: IntoIterator<Item = &'a EventId>,
    {
        let blacklist: RelayBlacklist = self.blacklist();
        blacklist.remove_ids(ids).await;

        // TODO: update mute list event?
    }

    /// Mute [PublicKey]s
    ///
    /// Add [PublicKey]s to blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    pub async fn mute_public_keys<I>(&self, public_keys: I)
    where
        I: IntoIterator<Item = PublicKey>,
    {
        let blacklist: RelayBlacklist = self.blacklist();
        blacklist.add_public_keys(public_keys).await;

        // TODO: create/update mute list event?
    }

    /// Unmute [PublicKey]s
    ///
    /// Remove [PublicKey]s from blacklist
    ///
    /// <div class="warning">Mute list event is not currently created/updated!</div>
    pub async fn unmute_public_keys<'a, I>(&self, public_keys: I)
    where
        I: IntoIterator<Item = &'a PublicKey>,
    {
        let blacklist: RelayBlacklist = self.blacklist();
        blacklist.remove_public_keys(public_keys).await;

        // TODO: update mute list event?
    }

    /// Completely shutdown [`Client`]
    #[inline]
    pub async fn shutdown(self) -> Result<(), Error> {
        Ok(self.pool.shutdown().await?)
    }

    /// Get new notification listener
    ///
    /// <div class="warning">When you call this method, you subscribe to the notifications channel from that precise moment. Anything received by relay/s before that moment is not included in the channel!</div>
    #[inline]
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.pool.notifications()
    }

    /// Get relays
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

    /// Add new relay
    ///
    /// Return `false` if the relay already exists.
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    ///
    /// This method use previously set or default [Options] to configure the [Relay] (ex. set proxy, set min POW, set relay limits, ...).
    /// To use custom [RelayOptions], check `Client::add_relay_with_opts`.
    ///
    /// Connection is **NOT** automatically started with relay, remember to call `client.connect()`!
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client.add_relay("wss://relay.nostr.info").await.unwrap();
    /// client.add_relay("wss://relay.damus.io").await.unwrap();
    ///
    /// client.connect().await;
    /// # }
    /// ```
    pub async fn add_relay<U>(&self, url: U) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let url: Url = url.try_into_url().map_err(pool::Error::from)?;
        let opts: RelayOptions = RelayOptions::new();

        // Set connection mode
        #[cfg(not(target_arch = "wasm32"))]
        let opts: RelayOptions = match self.opts.connection.mode {
            ConnectionMode::Direct => opts,
            ConnectionMode::Proxy(..) => match self.opts.connection.target {
                ConnectionTarget::All => opts.connection_mode(self.opts.connection.mode),
                ConnectionTarget::Onion => {
                    let domain: &str = url.domain().unwrap_or_default();

                    if domain.ends_with(".onion") {
                        opts.connection_mode(self.opts.connection.mode)
                    } else {
                        opts
                    }
                }
            },
            #[cfg(feature = "tor")]
            ConnectionMode::Tor => match self.opts.connection.target {
                ConnectionTarget::All => opts.connection_mode(self.opts.connection.mode),
                ConnectionTarget::Onion => {
                    let domain: &str = url.domain().unwrap_or_default();

                    if domain.ends_with(".onion") {
                        opts.connection_mode(self.opts.connection.mode)
                    } else {
                        opts
                    }
                }
            },
        };

        // Set min POW difficulty and limits
        let opts: RelayOptions = opts
            .pow(self.opts.get_min_pow_difficulty())
            .limits(self.opts.relay_limits.clone())
            .max_avg_latency(self.opts.max_avg_latency);

        // Add relay
        let added: bool = self.add_relay_with_opts::<&Url>(&url, opts).await?;

        if added && self.opts.autoconnect {
            self.connect_relay::<Url>(url).await?;
        }

        Ok(added)
    }

    /// Add new relay with custom [`RelayOptions`]
    ///
    /// Return `false` if the relay already exists.
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    ///
    /// Note: **this method ignore the options set in [`Options`]**.
    ///
    /// Connection is **NOT** automatically started with relay, remember to call `client.connect()`!
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    /// let mode = ConnectionMode::Proxy(addr);
    /// let opts = RelayOptions::new()
    ///     .connection_mode(mode)
    ///     .write(false)
    ///     .retry_sec(11);
    /// client
    ///     .add_relay_with_opts("wss://relay.nostr.info", opts)
    ///     .await
    ///     .unwrap();
    ///
    /// client.connect().await;
    /// # }
    /// ```
    #[inline]
    pub async fn add_relay_with_opts<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.add_relay(url, opts).await?)
    }

    /// Add multiple relays
    ///
    /// If are set pool subscriptions, the new added relay will inherit them. Use `subscribe_to` method instead of `subscribe`,
    /// to avoid to set pool subscriptions.
    ///
    /// Connection is **NOT** automatically started with relays, remember to call `client.connect()`!
    #[inline]
    pub async fn add_relays<I, U>(&self, relays: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        for url in relays.into_iter() {
            self.add_relay(url).await?;
        }
        Ok(())
    }

    /// Disconnect and remove relay
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client.remove_relay("wss://relay.nostr.info").await.unwrap();
    /// # }
    /// ```
    #[inline]
    pub async fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.remove_relay(url).await?)
    }

    /// Disconnect and remove all relays
    #[inline]
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        Ok(self.pool.remove_all_relays().await?)
    }

    /// Connect to a previously added relay
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .connect_relay("wss://relay.nostr.info")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
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
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .disconnect_relay("wss://relay.nostr.info")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn disconnect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let relay = self.relay(url).await?;
        relay.disconnect().await?;
        Ok(())
    }

    /// Connect to all added relays
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client.connect().await;
    /// # }
    /// ```
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
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client.disconnect().await.unwrap();
    /// # }
    /// ```
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

    /// Subscribe to filters to all connected relays
    ///
    /// This method create a new subscription. None of the previous subscriptions will be edited/closed when you call this!
    /// So remember to unsubscribe when you no longer need it. You can get all your active **pool** (non-auto-closing) subscriptions
    /// by calling `client.subscriptions().await`.
    ///
    /// # Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeAutoCloseOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let subscription = Filter::new()
    ///     .pubkeys(vec![my_keys.public_key()])
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
        let send_opts: RelaySendOptions = self.opts.get_wait_for_subscription();
        let opts: SubscribeOptions = SubscribeOptions::default()
            .close_on(opts)
            .send_opts(send_opts);
        Ok(self.pool.subscribe(filters, opts).await?)
    }

    /// Subscribe to filters with custom [SubscriptionId] to all connected relays
    ///
    /// # Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the [SubscribeAutoCloseOptions].
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let id = SubscriptionId::new("myid");
    /// let subscription = Filter::new()
    ///     .pubkeys(vec![my_keys.public_key()])
    ///     .since(Timestamp::now());
    ///
    /// // Subscribe
    /// client
    ///     .subscribe_with_id(id, vec![subscription], None)
    ///     .await?;
    ///
    /// // Auto-closing subscription
    /// let id = SubscriptionId::generate();
    /// let subscription = Filter::new().kind(Kind::TextNote).limit(10);
    /// let opts = SubscribeAutoCloseOptions::default().filter(FilterOptions::ExitOnEOSE);
    /// client
    ///     .subscribe_with_id(id, vec![subscription], Some(opts))
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_with_id(
        &self,
        id: SubscriptionId,
        filters: Vec<Filter>,
        opts: Option<SubscribeAutoCloseOptions>,
    ) -> Result<Output<()>, Error> {
        let send_opts: RelaySendOptions = self.opts.get_wait_for_subscription();
        let opts: SubscribeOptions = SubscribeOptions::default()
            .close_on(opts)
            .send_opts(send_opts);
        Ok(self.pool.subscribe_with_id(id, filters, opts).await?)
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
        let send_opts: RelaySendOptions = self.opts.get_wait_for_subscription();
        let opts: SubscribeOptions = SubscribeOptions::default()
            .close_on(opts)
            .send_opts(send_opts);
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
        let send_opts: RelaySendOptions = self.opts.get_wait_for_subscription();
        let opts: SubscribeOptions = SubscribeOptions::default()
            .close_on(opts)
            .send_opts(send_opts);
        Ok(self
            .pool
            .subscribe_with_id_to(urls, id, filters, opts)
            .await?)
    }

    /// Unsubscribe
    #[inline]
    pub async fn unsubscribe(&self, id: SubscriptionId) {
        let opts: RelaySendOptions = self.opts.get_wait_for_subscription();
        self.pool.unsubscribe(id, opts).await;
    }

    /// Unsubscribe from all subscriptions
    #[inline]
    pub async fn unsubscribe_all(&self) {
        let opts: RelaySendOptions = self.opts.get_wait_for_subscription();
        self.pool.unsubscribe_all(opts).await;
    }

    /// Get events of filters
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::time::Duration;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let subscription = Filter::new()
    ///     .pubkeys(vec![my_keys.public_key()])
    ///     .since(Timestamp::now());
    ///
    /// let timeout = Some(Duration::from_secs(10));
    /// let _events = client
    ///     .get_events_of(vec![subscription], EventSource::both(timeout))
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    #[inline]
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        source: EventSource,
    ) -> Result<Vec<Event>, Error> {
        match source {
            EventSource::Database => Ok(self.database().query(filters, Order::Desc).await?),
            EventSource::Relays {
                timeout,
                specific_relays,
            } => match specific_relays {
                Some(urls) => self.get_events_from(urls, filters, timeout).await,
                None => {
                    let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
                    Ok(self
                        .pool
                        .get_events_of(filters, timeout, FilterOptions::ExitOnEOSE)
                        .await?)
                }
            },
            EventSource::Both {
                timeout,
                specific_relays,
            } => {
                // Check how many filters are passed and return the limit
                let limit: Option<usize> = match (filters.len(), filters.first()) {
                    (1, Some(filter)) => filter.limit,
                    _ => None,
                };

                let stored = self.database().query(filters.clone(), Order::Desc).await?;
                let mut events: BTreeSet<Event> = stored.into_iter().collect();

                let mut stream: ReceiverStream<Event> = match specific_relays {
                    Some(urls) => self.stream_events_from(urls, filters, timeout).await?,
                    None => self.stream_events_of(filters, timeout).await?,
                };

                while let Some(event) = stream.next().await {
                    events.insert(event);
                }

                let iter: Rev<IntoIter<Event>> = events.into_iter().rev();

                // Check limit
                match limit {
                    Some(limit) => Ok(iter.take(limit).collect()),
                    None => Ok(iter.collect()),
                }
            }
        }
    }

    /// Get events of filters with [`FilterOptions`]
    ///
    /// If timeout is set to `None`, the default from [`Options`] will be used.
    #[inline]
    #[deprecated(since = "0.34.0", note = "Use `client.pool().get_events_of(..)`.")]
    pub async fn get_events_of_with_opts(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
        Ok(self.pool.get_events_of(filters, timeout, opts).await?)
    }

    /// Get events of filters from specific relays
    #[inline]
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
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
        Ok(self
            .pool
            .get_events_from(urls, filters, timeout, FilterOptions::ExitOnEOSE)
            .await?)
    }

    /// Stream events of filters
    #[inline]
    pub async fn stream_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<ReceiverStream<Event>, Error> {
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
        Ok(self
            .pool
            .stream_events_of(filters, timeout, FilterOptions::ExitOnEOSE)
            .await?)
    }

    /// Stream events of filters from **specific relays**
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

    /// Send client message to **all relays**
    #[inline]
    pub async fn send_msg(&self, msg: ClientMessage) -> Result<Output<()>, Error> {
        let opts: RelaySendOptions = self.opts.get_wait_for_send();
        Ok(self.pool.send_msg(msg, opts).await?)
    }

    /// Batch send client messages to **all relays**
    #[inline]
    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        Ok(self.pool.batch_msg(msgs, opts).await?)
    }

    /// Send client message to a **specific relays**
    #[inline]
    pub async fn send_msg_to<I, U>(&self, urls: I, msg: ClientMessage) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let opts: RelaySendOptions = self.opts.get_wait_for_send();
        Ok(self.pool.send_msg_to(urls, msg, opts).await?)
    }

    /// Batch send client messages to **specific relays**
    #[inline]
    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.batch_msg_to(urls, msgs, opts).await?)
    }

    /// Send event to **all relays**
    ///
    /// This method will wait for the `OK` message from the relay.
    /// If you not want to wait for the `OK` message, use `send_msg` method instead.
    #[inline]
    pub async fn send_event(&self, event: Event) -> Result<Output<EventId>, Error> {
        let opts: RelaySendOptions = self.opts.get_wait_for_send();
        Ok(self.pool.send_event(event, opts).await?)
    }

    /// Send multiple [`Event`] at once to **all relays**.
    #[inline]
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error> {
        Ok(self.pool.batch_event(events, opts).await?)
    }

    /// Send event to **specific relays**.
    ///
    /// This method will wait for the `OK` message from the relay.
    /// If you not want to wait for the `OK` message, use `send_msg` method instead.
    #[inline]
    pub async fn send_event_to<I, U>(&self, urls: I, event: Event) -> Result<Output<EventId>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let opts: RelaySendOptions = self.opts.get_wait_for_send();
        Ok(self.pool.send_event_to(urls, event, opts).await?)
    }

    /// Send multiple [`Event`] at once to **specific relays**.
    #[inline]
    pub async fn batch_event_to<I, U>(
        &self,
        urls: I,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<Output<()>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.batch_event_to(urls, events, opts).await?)
    }

    /// Signs the [`EventBuilder`] into an [`Event`] using the [`NostrSigner`]
    pub async fn sign_event_builder(&self, builder: EventBuilder) -> Result<Event, Error> {
        let signer = self.signer().await?;

        let public_key = signer.public_key().await?;
        let difficulty: u8 = self.opts.get_difficulty();
        let unsigned = if difficulty > 0 {
            builder.to_unsigned_pow_event(public_key, difficulty)
        } else {
            builder.to_unsigned_event(public_key)
        };

        Ok(signer.sign_event(unsigned).await?)
    }

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to **all relays**.
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

    /// Take an [`EventBuilder`], sign it by using the [`NostrSigner`] and broadcast to **specific relays**.
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

    /// Fetch public key metadata from database, falling back to connected relays.
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
        let events: Vec<Event> = self
            .get_events_of(vec![filter], EventSource::both(timeout))
            .await?;
        match events.first() {
            Some(event) => Ok(Metadata::try_from(event)?),
            None => Err(Error::MetadataNotFound),
        }
    }

    /// Get cached public key metadata.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub async fn get_metadata(&self, public_key: PublicKey) -> Result<Metadata, Error> {
        let filter: Filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        let events: Vec<Event> = self
            .get_events_of(vec![filter], EventSource::Database)
            .await?;
        match events.first() {
            Some(event) => Ok(Metadata::try_from(event)?),
            None => Err(Error::MetadataNotFound),
        }
    }

    /// Fetch public key metadata from database, falling back to connected relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[deprecated]
    pub async fn metadata(&self, public_key: PublicKey) -> Result<Metadata, Error> {
        let filter: Filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        let events: Vec<Event> = self
            .get_events_of(vec![filter], EventSource::both(None))
            .await?;
        match events.first() {
            Some(event) => Ok(Metadata::from_json(&event.content)?),
            None => Err(Error::MetadataNotFound),
        }
    }

    /// Update metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
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
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// client
    ///     .publish_text_note("My first text note from rust-nostr!", [])
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
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
        let public_key = signer.public_key().await?;
        let filter: Filter = Filter::new()
            .author(public_key)
            .kind(Kind::ContactList)
            .limit(1);
        Ok(vec![filter])
    }

    /// Get contact list from database, falling back to connected relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::time::Duration;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let timeout = Duration::from_secs(10);
    /// let _list = client.get_contact_list(Some(timeout)).await.unwrap();
    /// # }
    /// ```
    pub async fn get_contact_list(&self, timeout: Option<Duration>) -> Result<Vec<Contact>, Error> {
        let mut contact_list: Vec<Contact> = Vec::new();
        let filters: Vec<Filter> = self.get_contact_list_filters().await?;
        let events: Vec<Event> = self
            .get_events_of(filters, EventSource::both(timeout))
            .await?;

        // Get first event (result of `get_events_of` is sorted DESC by timestamp)
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

    /// Get contact list public keys from database, falling back to connected relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn get_contact_list_public_keys(
        &self,
        timeout: Option<Duration>,
    ) -> Result<Vec<PublicKey>, Error> {
        let mut pubkeys: Vec<PublicKey> = Vec::new();
        let filters: Vec<Filter> = self.get_contact_list_filters().await?;
        let events: Vec<Event> = self
            .get_events_of(filters, EventSource::both(timeout))
            .await?;

        for event in events.into_iter() {
            pubkeys.extend(event.public_keys());
        }

        Ok(pubkeys)
    }

    /// Get contact list [`Metadata`] from database, falling back to connected relays.
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
            let events: Vec<Event> = self
                .get_events_of(filters, EventSource::both(timeout))
                .await?;
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
    ///
    /// # Example
    /// ```rust,no_run
    /// use std::str::FromStr;
    ///
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let event =
    ///     Event::from_json(r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#)
    ///         .unwrap();
    ///
    /// client.like(&event).await.unwrap();
    /// # }
    /// ```
    #[inline]
    pub async fn like(&self, event: &Event) -> Result<Output<EventId>, Error> {
        self.reaction(event, "+").await
    }

    /// Disike event
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
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let event =
    ///     Event::from_json(r#"{"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","created_at":1640839235,"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","kind":4,"pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd","tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]]}"#)
    ///         .unwrap();
    ///
    /// client.dislike(&event).await.unwrap();
    /// # }
    /// ```
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
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
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
        relay.auth(event, self.opts.get_wait_for_send()).await?;

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

    /// Build gift wrap event
    #[cfg(feature = "nip59")]
    async fn internal_gift_wrap(
        &self,
        receiver: &PublicKey,
        rumor: EventBuilder,
        expiration: Option<Timestamp>,
    ) -> Result<Event, Error> {
        // Compose rumor
        let signer: NostrSigner = self.signer().await?;
        let public_key: PublicKey = signer.public_key().await?;
        let rumor = rumor.to_unsigned_event(public_key);

        // Compose seal
        // TODO: use directly the `EventBuilder::seal` constructor
        let content: String = signer.nip44_encrypt(receiver, rumor.as_json()).await?;
        let seal: EventBuilder = EventBuilder::new(Kind::Seal, content, [])
            .custom_created_at(Timestamp::tweaked(nip59::RANGE_RANDOM_TIMESTAMP_TWEAK));
        let seal: Event = self.sign_event_builder(seal).await?;

        // Compose gift wrap
        Ok(EventBuilder::gift_wrap_from_seal(
            receiver, &seal, expiration,
        )?)
    }

    /// Construct Gift Wrap and send to all relays
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
        let gift_wrap: Event = self.internal_gift_wrap(receiver, rumor, expiration).await?;
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
        let gift_wrap: Event = self.internal_gift_wrap(receiver, rumor, expiration).await?;
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
        let signer: NostrSigner = self.signer().await?;
        Ok(signer.unwrap_gift_wrap(gift_wrap).await?)
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

    /// Negentropy reconciliation with all connected relays
    ///
    /// <https://github.com/hoytech/negentropy>
    #[inline]
    pub async fn reconcile(
        &self,
        filter: Filter,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        Ok(self.pool.reconcile(filter, opts).await?)
    }

    /// Negentropy reconciliation with specified relays
    ///
    /// <https://github.com/hoytech/negentropy>
    #[inline]
    pub async fn reconcile_with<I, U>(
        &self,
        urls: I,
        filter: Filter,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.reconcile_with(urls, filter, opts).await?)
    }

    /// Negentropy reconciliation with all relays and custom items
    #[inline]
    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error> {
        Ok(self.pool.reconcile_with_items(filter, items, opts).await?)
    }

    /// Negentropy reconciliation with specified relays and custom items
    ///
    /// <https://github.com/hoytech/negentropy>
    #[inline]
    pub async fn reconcile_advanced<I, U>(
        &self,
        urls: I,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<Output<Reconciliation>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self
            .pool
            .reconcile_advanced(urls, filter, items, opts)
            .await?)
    }

    /// Handle notifications
    #[inline]
    pub async fn handle_notifications<F, Fut>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(RelayPoolNotification) -> Fut,
        Fut: Future<Output = Result<bool>>,
    {
        Ok(self.pool.handle_notifications(func).await?)
    }
}
