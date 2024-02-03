// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Client

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::thread;
use nostr::event::builder::Error as EventBuilderError;
use nostr::prelude::*;
use nostr::types::metadata::Error as MetadataError;
use nostr_database::DynNostrDatabase;
use nostr_sdk_pool::pool::{self, Error as RelayPoolError, RelayPool};
use nostr_sdk_pool::{
    Error as RelayError, FilterOptions, NegentropyOptions, Relay, RelayOptions,
    RelayPoolNotification, RelaySendOptions,
};
use tokio::sync::{broadcast, RwLock};

#[cfg(feature = "blocking")]
pub mod blocking;
pub mod builder;
pub mod options;
pub mod signer;
#[cfg(feature = "nip57")]
pub mod zapper;

pub use self::builder::ClientBuilder;
pub use self::options::Options;
#[cfg(feature = "nip46")]
pub use self::signer::nip46::Nip46Signer;
pub use self::signer::{ClientSigner, ClientSignerType};
#[cfg(feature = "nip57")]
pub use self::zapper::{ClientZapper, ZapDetails, ZapEntity};

/// [`Client`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Keys error
    #[error(transparent)]
    Keys(#[from] nostr::key::Error),
    /// Url parse error
    #[error("impossible to parse URL: {0}")]
    Url(#[from] nostr::types::url::ParseError),
    /// [`RelayPool`] error
    #[error("relay pool error: {0}")]
    RelayPool(#[from] RelayPoolError),
    /// [`Relay`] error
    #[error("relay error: {0}")]
    Relay(#[from] RelayError),
    /// [`EventBuilder`] error
    #[error("event builder error: {0}")]
    EventBuilder(#[from] EventBuilderError),
    /// Unsigned event error
    #[error("unsigned event error: {0}")]
    UnsignedEvent(#[from] nostr::event::unsigned::Error),
    /// Secp256k1 error
    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] nostr::secp256k1::Error),
    /// Hex error
    #[error("hex decoding error: {0}")]
    Hex(#[from] nostr::hashes::hex::Error),
    /// Metadata error
    #[error(transparent)]
    Metadata(#[from] MetadataError),
    /// Notification Handler error
    #[error("notification handler error: {0}")]
    Handler(String),
    /// Signer not configured
    #[error("signer not configured")]
    SignerNotConfigured,
    /// Signer not configured
    #[error("wrong signer: expected={expected}, found={found}")]
    WrongSigner {
        /// Expected client signer type
        expected: ClientSignerType,
        /// Found client signer type
        found: ClientSignerType,
    },
    /// Zapper not configured
    #[cfg(feature = "nip57")]
    #[error("zapper not configured")]
    ZapperNotConfigured,
    /// NIP04 error
    #[cfg(feature = "nip04")]
    #[error(transparent)]
    NIP04(#[from] nostr::nips::nip04::Error),
    /// NIP07 error
    #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
    #[error(transparent)]
    NIP07(#[from] nostr::nips::nip07::Error),
    /// JSON error
    #[cfg(feature = "nip46")]
    #[error(transparent)]
    JSON(#[from] nostr::serde_json::Error),
    /// NIP44 error
    #[cfg(feature = "nip44")]
    #[error(transparent)]
    NIP44(#[from] nip44::Error),
    /// NIP46 error
    #[cfg(feature = "nip46")]
    #[error(transparent)]
    NIP46(#[from] nostr::nips::nip46::Error),
    /// NIP47 error
    #[cfg(feature = "nip47")]
    #[error(transparent)]
    NIP47(#[from] nostr::nips::nip47::Error),
    /// NIP47 Error Code
    #[cfg(feature = "nip47")]
    #[error("{0}")]
    NIP47ErrorCode(NIP47Error),
    /// NIP47 Error Code
    #[cfg(feature = "nip47")]
    #[error("Unexpected NIP47 result: {0}")]
    NIP47Unexpected(String),
    /// NIP57 error
    #[cfg(feature = "nip57")]
    #[error(transparent)]
    NIP57(#[from] nostr::nips::nip57::Error),
    /// LNURL Pay
    #[cfg(feature = "nip57")]
    #[error(transparent)]
    LnUrlPay(#[from] lnurl_pay::Error),
    /// WebLN error
    #[cfg(all(feature = "webln", target_arch = "wasm32"))]
    #[error(transparent)]
    WebLN(#[from] webln::Error),
    /// Generic NIP46 error
    #[cfg(feature = "nip46")]
    #[error("generic error")]
    Generic,
    /// NIP46 response error
    #[cfg(feature = "nip46")]
    #[error("response error: {0}")]
    Response(String),
    /// Signer public key not found
    #[cfg(feature = "nip46")]
    #[error("signer public key not found")]
    SignerPublicKeyNotFound,
    /// Timeout
    #[error("timeout")]
    Timeout,
    /// Response not match to the request
    #[error("response not match to the request")]
    ResponseNotMatchRequest,
    /// Event not found
    #[error("event not found: {0}")]
    EventNotFound(EventId),
    /// Event not found
    #[error("event not found")]
    GenericEventNotFound,
    /// Impossible to zap
    #[error("impossible to send zap: {0}")]
    ImpossibleToZap(String),
    /// Not supported yet
    #[error("{0}")]
    Unsupported(String),
}

#[cfg(feature = "nip44")]
impl Error {
    fn unsupported<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Unsupported(message.into())
    }
}

/// Nostr client
#[derive(Debug, Clone)]
pub struct Client {
    pool: RelayPool,
    signer: Arc<RwLock<Option<ClientSigner>>>,
    #[cfg(feature = "nip57")]
    zapper: Arc<RwLock<Option<ClientZapper>>>,
    opts: Options,
    dropped: Arc<AtomicBool>,
}

impl Default for Client {
    fn default() -> Self {
        ClientBuilder::new().build()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if self.opts.shutdown_on_drop {
            if self.dropped.load(Ordering::SeqCst) {
                tracing::warn!("Client already dropped");
            } else {
                tracing::debug!("Dropping the Client...");
                self.dropped.store(true, Ordering::SeqCst);
                let client: Client = self.clone();
                thread::spawn(async move {
                    client
                        .shutdown()
                        .await
                        .expect("Impossible to drop the client")
                });
            }
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
    pub fn new<S>(signer: S) -> Self
    where
        S: Into<ClientSigner>,
    {
        Self::with_opts(signer, Options::default())
    }

    /// Create a new [`Client`] with [`Options`]
    ///
    /// To create a [`Client`] with custom [`Options`] and without any signer use `ClientBuilder::new().opts(opts).build()`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// let my_keys = Keys::generate();
    /// let opts = Options::new().wait_for_send(true);
    /// let client = Client::with_opts(&my_keys, opts);
    /// ```
    pub fn with_opts<S>(signer: S, opts: Options) -> Self
    where
        S: Into<ClientSigner>,
    {
        ClientBuilder::new().signer(signer).opts(opts).build()
    }

    /// Compose [`Client`] from [`ClientBuilder`]
    pub fn from_builder(builder: ClientBuilder) -> Self {
        Self {
            pool: RelayPool::with_database(builder.opts.pool, builder.database),
            signer: Arc::new(RwLock::new(builder.signer)),
            #[cfg(feature = "nip57")]
            zapper: Arc::new(RwLock::new(builder.zapper)),
            opts: builder.opts,
            dropped: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Update default difficulty for new [`Event`]
    pub fn update_difficulty(&self, difficulty: u8) {
        self.opts.update_difficulty(difficulty);
    }

    /// Get current client signer
    ///
    /// Rise error if it not set.
    pub async fn signer(&self) -> Result<ClientSigner, Error> {
        let signer = self.signer.read().await;
        signer.clone().ok_or(Error::SignerNotConfigured)
    }

    /// Set client signer
    pub async fn set_signer(&self, signer: Option<ClientSigner>) {
        let mut s = self.signer.write().await;
        *s = signer;
    }

    /// Get current client zapper
    ///
    /// Rise error if it not set.
    #[cfg(feature = "nip57")]
    pub async fn zapper(&self) -> Result<ClientZapper, Error> {
        let zapper = self.zapper.read().await;
        zapper.clone().ok_or(Error::ZapperNotConfigured)
    }

    /// Set client zapper
    #[cfg(feature = "nip57")]
    pub async fn set_zapper(&self, zapper: Option<ClientZapper>) {
        let mut s = self.zapper.write().await;
        *s = zapper;
    }

    /// Get [`RelayPool`]
    pub fn pool(&self) -> RelayPool {
        self.pool.clone()
    }

    /// Get database
    pub fn database(&self) -> Arc<DynNostrDatabase> {
        self.pool.database()
    }

    /// Start a previously stopped client
    pub async fn start(&self) {
        self.pool.start();
        self.connect().await;
    }

    /// Stop the client
    ///
    /// Disconnect all relays and set their status to `RelayStatus::Stopped`.
    pub async fn stop(&self) -> Result<(), Error> {
        Ok(self.pool.stop().await?)
    }

    /// Check if [`RelayPool`] is running
    pub fn is_running(&self) -> bool {
        self.pool.is_running()
    }

    /// Completely shutdown [`Client`]
    pub async fn shutdown(self) -> Result<(), Error> {
        Ok(self.pool.clone().shutdown().await?)
    }

    /// Get new notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.pool.notifications()
    }

    /// Get relays
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        self.pool.relays().await
    }

    /// Get a previously added [`Relay`]
    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.relay(url).await?)
    }

    /// Add new relay
    ///
    /// This method **NOT** automatically start connection with relay!
    ///
    /// Return `false` if the relay already exists.
    ///
    /// To use a proxy, see `Client::add_relay_with_opts`.
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
        #[cfg(not(target_arch = "wasm32"))]
        let opts: RelayOptions = RelayOptions::new().proxy(self.opts.proxy);
        #[cfg(target_arch = "wasm32")]
        let opts: RelayOptions = RelayOptions::new();
        self.add_relay_with_opts(url, opts).await
    }

    /// Add new relay with [`RelayOptions`]
    ///
    /// This method **NOT** automatically start connection with relay!
    ///
    /// Return `false` if the relay already exists.
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
    /// let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));
    /// let opts = RelayOptions::new().proxy(proxy).write(false).retry_sec(11);
    /// client
    ///     .add_relay_with_opts("wss://relay.nostr.info", opts)
    ///     .await
    ///     .unwrap();
    ///
    /// client.connect().await;
    /// # }
    /// ```
    pub async fn add_relay_with_opts<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.add_relay(url, opts).await?)
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
    pub async fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        self.pool.remove_relay(url).await?;
        Ok(())
    }

    /// Add multiple relays
    ///
    /// This method **NOT** automatically start connection with relays!
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
    pub async fn connect_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let relay: Relay = self.relay(url).await?;
        self.pool
            .connect_relay(&relay, self.opts.connection_timeout)
            .await;
        Ok(())
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
        self.pool.disconnect_relay(&relay).await?;
        Ok(())
    }

    /// Connect relays
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
    pub async fn connect(&self) {
        self.pool.connect(self.opts.connection_timeout).await;
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
    pub async fn disconnect(&self) -> Result<(), Error> {
        Ok(self.pool.disconnect().await?)
    }

    /// Subscribe to filters
    ///
    /// # Example
    /// ```rust,no_run
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
    /// client.subscribe(vec![subscription]).await;
    /// # }
    /// ```
    pub async fn subscribe(&self, filters: Vec<Filter>) {
        let opts: RelaySendOptions = self.opts.get_wait_for_subscription();
        self.pool.subscribe(filters, opts).await;
    }

    /// Unsubscribe from filters
    pub async fn unsubscribe(&self) {
        let opts: RelaySendOptions = self.opts.get_wait_for_subscription();
        self.pool.unsubscribe(opts).await;
    }

    /// Get events of filters
    ///
    /// If timeout is set to `None`, the default from [`Options`] will be used.
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
    /// let timeout = Duration::from_secs(10);
    /// let _events = client
    ///     .get_events_of(vec![subscription], Some(timeout))
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        self.get_events_of_with_opts(filters, timeout, FilterOptions::ExitOnEOSE)
            .await
    }

    /// Get events of filters with [`FilterOptions`]
    ///
    /// If timeout is set to `None`, the default from [`Options`] will be used.
    pub async fn get_events_of_with_opts(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let timeout: Duration = match timeout {
            Some(t) => t,
            None => self.opts.timeout,
        };
        Ok(self.pool.get_events_of(filters, timeout, opts).await?)
    }

    /// Get events of filters from specific relays
    ///
    /// Get events both from **local database** and **relays**
    ///
    /// If no relay is specified, will be queried only the database.
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

    /// Request events of filters
    ///
    /// All events will be received on notification listener (`client.notifications()`)
    /// until the EOSE "end of stored events" message is received from the relay.
    ///
    /// If timeout is set to `None`, the default from [`Options`] will be used.
    pub async fn req_events_of(&self, filters: Vec<Filter>, timeout: Option<Duration>) {
        self.req_events_of_with_opts(filters, timeout, FilterOptions::ExitOnEOSE)
            .await
    }

    /// Request events of filters with [`FilterOptions`]
    ///
    /// If timeout is set to `None`, the default from [`Options`] will be used.
    pub async fn req_events_of_with_opts(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
        opts: FilterOptions,
    ) {
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
        self.pool.req_events_of(filters, timeout, opts).await;
    }

    /// Request events of filters from specific relays
    ///
    /// All events will be received on notification listener (`client.notifications()`)
    /// until the EOSE "end of stored events" message is received from the relay.
    ///
    /// If timeout is set to `None`, the default from [`Options`] will be used.
    pub async fn req_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let timeout: Duration = timeout.unwrap_or(self.opts.timeout);
        self.pool
            .req_events_from(urls, filters, timeout, FilterOptions::ExitOnEOSE)
            .await?;
        Ok(())
    }

    /// Send client message to **all relays**
    pub async fn send_msg(&self, msg: ClientMessage) -> Result<(), Error> {
        let opts: RelaySendOptions = self.opts.get_wait_for_send();
        Ok(self.pool.send_msg(msg, opts).await?)
    }

    /// Batch send client messages to **all relays**
    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        Ok(self.pool.batch_msg(msgs, opts).await?)
    }

    /// Send client message to a **specific relays**
    pub async fn send_msg_to<I, U>(&self, urls: I, msg: ClientMessage) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let opts: RelaySendOptions = self.opts.get_wait_for_send();
        Ok(self.pool.send_msg_to(urls, msg, opts).await?)
    }

    /// Batch send client messages to **specific relays**
    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error>
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
    pub async fn send_event(&self, event: Event) -> Result<EventId, Error> {
        let opts: RelaySendOptions = self.opts.get_wait_for_send();
        Ok(self.pool.send_event(event, opts).await?)
    }

    /// Send multiple [`Event`] at once to **all relays**.
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        Ok(self.pool.batch_event(events, opts).await?)
    }

    /// Send event to **specific relays**.
    ///
    /// This method will wait for the `OK` message from the relay.
    /// If you not want to wait for the `OK` message, use `send_msg` method instead.
    pub async fn send_event_to<I, U>(&self, urls: I, event: Event) -> Result<EventId, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let opts: RelaySendOptions = self.opts.get_wait_for_send();
        Ok(self.pool.send_event_to(urls, event, opts).await?)
    }

    /// Send multiple [`Event`] at once to **specific relays**.
    pub async fn batch_event_to<I, U>(
        &self,
        urls: I,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        Ok(self.pool.batch_event_to(urls, events, opts).await?)
    }

    /// Signs the [`EventBuilder`] into an [`Event`] using the [`ClientSigner`]
    pub async fn sign_event_builder(&self, builder: EventBuilder) -> Result<Event, Error> {
        match self.signer().await? {
            ClientSigner::Keys(keys) => {
                let difficulty: u8 = self.opts.get_difficulty();
                if difficulty > 0 {
                    Ok(builder.to_pow_event(&keys, difficulty)?)
                } else {
                    Ok(builder.to_event(&keys)?)
                }
            }
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(nip07) => {
                let public_key: XOnlyPublicKey = nip07.get_public_key().await?;
                let unsigned = {
                    let difficulty: u8 = self.opts.get_difficulty();
                    if difficulty > 0 {
                        builder.to_unsigned_pow_event(public_key, difficulty)
                    } else {
                        builder.to_unsigned_event(public_key)
                    }
                };
                Ok(nip07.sign_event(unsigned).await?)
            }
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(nip46) => {
                let signer_public_key: XOnlyPublicKey = nip46
                    .signer_public_key()
                    .await
                    .ok_or(Error::SignerPublicKeyNotFound)?;
                let unsigned = {
                    let difficulty: u8 = self.opts.get_difficulty();
                    if difficulty > 0 {
                        builder.to_unsigned_pow_event(signer_public_key, difficulty)
                    } else {
                        builder.to_unsigned_event(signer_public_key)
                    }
                };
                let res: nip46::Response = self
                    .send_req_to_signer(
                        nip46::Request::SignEvent(unsigned),
                        self.opts.nip46_timeout,
                    )
                    .await?;
                if let nip46::Response::SignEvent(event) = res {
                    Ok(event)
                } else {
                    Err(Error::ResponseNotMatchRequest)
                }
            }
        }
    }

    /// Take an [`EventBuilder`], sign it by using the [`ClientSigner`] and broadcast to **all relays**.
    ///
    /// Rise an error if the [`ClientSigner`] is not set.
    pub async fn send_event_builder(&self, builder: EventBuilder) -> Result<EventId, Error> {
        let event: Event = self.sign_event_builder(builder).await?;
        self.send_event(event).await
    }

    /// Take an [`EventBuilder`], sign it by using the [`ClientSigner`] and broadcast to **specific relays**.
    ///
    /// Rise an error if the [`ClientSigner`] is not set.
    pub async fn send_event_builder_to<I, U>(
        &self,
        urls: I,
        builder: EventBuilder,
    ) -> Result<EventId, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        pool::Error: From<<U as TryIntoUrl>::Err>,
    {
        let event: Event = self.sign_event_builder(builder).await?;
        self.send_event_to(urls, event).await
    }

    /// NIP44 encryption with [ClientSigner]
    #[cfg(feature = "nip44")]
    pub async fn nip44_encrypt<T>(
        &self,
        public_key: XOnlyPublicKey,
        content: T,
    ) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        match self.signer().await? {
            ClientSigner::Keys(keys) => Ok(nip44::encrypt(
                &keys.secret_key()?,
                &public_key,
                content,
                nip44::Version::default(),
            )?),
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(..) => Err(Error::unsupported(
                "NIP44 encryption not supported with NIP07 signer yet!",
            )),
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(..) => Err(Error::unsupported(
                "NIP44 encryption not supported with NIP46 signer yet!",
            )),
        }
    }

    /// NIP44 decryption with [ClientSigner]
    #[cfg(feature = "nip44")]
    pub async fn nip44_decrypt<T>(
        &self,
        public_key: XOnlyPublicKey,
        payload: T,
    ) -> Result<String, Error>
    where
        T: AsRef<[u8]>,
    {
        match self.signer().await? {
            ClientSigner::Keys(keys) => {
                Ok(nip44::decrypt(&keys.secret_key()?, &public_key, payload)?)
            }
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(..) => Err(Error::unsupported(
                "NIP44 decryption not supported with NIP07 signer yet!",
            )),
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(..) => Err(Error::unsupported(
                "NIP44 decryption not supported with NIP46 signer yet!",
            )),
        }
    }

    /// Get public key metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub async fn metadata(&self, public_key: XOnlyPublicKey) -> Result<Metadata, Error> {
        let filter: Filter = Filter::new()
            .author(public_key)
            .kind(Kind::Metadata)
            .limit(1);
        let events: Vec<Event> = self.get_events_of(vec![filter], None).await?;
        match events.first() {
            Some(event) => Ok(Metadata::from_json(event.content())?),
            None => Ok(Metadata::default()),
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
    pub async fn set_metadata(&self, metadata: &Metadata) -> Result<EventId, Error> {
        let builder = EventBuilder::metadata(metadata);
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
    ///     .publish_text_note("My first text note from Nostr SDK!", [])
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn publish_text_note<S, I>(&self, content: S, tags: I) -> Result<EventId, Error>
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
    pub async fn set_contact_list<I>(&self, list: I) -> Result<EventId, Error>
    where
        I: IntoIterator<Item = Contact>,
    {
        let builder = EventBuilder::contact_list(list);
        self.send_event_builder(builder).await
    }

    async fn get_contact_list_filters(&self) -> Result<Vec<Filter>, Error> {
        let mut filter: Filter = Filter::new().kind(Kind::ContactList).limit(1);

        match self.signer().await? {
            ClientSigner::Keys(keys) => {
                filter = filter.author(keys.public_key());
            }
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(nip07) => {
                let public_key: XOnlyPublicKey = nip07.get_public_key().await?;
                filter = filter.author(public_key);
            }
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(nip46) => {
                let signer_public_key = nip46
                    .signer_public_key()
                    .await
                    .ok_or(Error::SignerPublicKeyNotFound)?;

                filter = filter.author(signer_public_key);
            }
        };

        Ok(vec![filter])
    }

    /// Get contact list
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
        let events: Vec<Event> = self.get_events_of(filters, timeout).await?;

        for event in events.into_iter() {
            for tag in event.into_iter_tags() {
                if let Tag::PublicKey {
                    public_key,
                    relay_url,
                    alias,
                    uppercase: false,
                } = tag
                {
                    contact_list.push(Contact::new(public_key, relay_url, alias))
                }
            }
        }

        Ok(contact_list)
    }

    /// Get contact list public keys
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/02.md>
    pub async fn get_contact_list_public_keys(
        &self,
        timeout: Option<Duration>,
    ) -> Result<Vec<XOnlyPublicKey>, Error> {
        let mut pubkeys: Vec<XOnlyPublicKey> = Vec::new();
        let filters: Vec<Filter> = self.get_contact_list_filters().await?;
        let events: Vec<Event> = self.get_events_of(filters, timeout).await?;

        for event in events.into_iter() {
            pubkeys.extend(event.public_keys());
        }

        Ok(pubkeys)
    }

    /// Get contact list [`Metadata`]
    pub async fn get_contact_list_metadata(
        &self,
        timeout: Option<Duration>,
    ) -> Result<HashMap<XOnlyPublicKey, Metadata>, Error> {
        let public_keys = self.get_contact_list_public_keys(timeout).await?;
        let mut contacts: HashMap<XOnlyPublicKey, Metadata> =
            public_keys.iter().map(|p| (*p, Metadata::new())).collect();

        let chunk_size: usize = self.opts.get_req_filters_chunk_size();
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
            let events: Vec<Event> = self.get_events_of(filters, timeout).await?;
            for event in events.into_iter() {
                let metadata = Metadata::from_json(event.content())?;
                if let Some(m) = contacts.get_mut(&event.author()) {
                    *m = metadata
                };
            }
        }

        Ok(contacts)
    }

    /// Send encrypted direct message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/04.md>
    ///
    /// # Example
    /// ```rust,no_run
    /// use nostr_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #   let my_keys = Keys::generate();
    /// #   let client = Client::new(&my_keys);
    /// let alice_pubkey = XOnlyPublicKey::from_bech32(
    ///     "npub14f8usejl26twx0dhuxjh9cas7keav9vr0v8nvtwtrjqx3vycc76qqh9nsy",
    /// )
    /// .unwrap();
    ///
    /// client
    ///     .send_direct_msg(alice_pubkey, "My first DM fro Nostr SDK!", None)
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    #[cfg(feature = "nip04")]
    pub async fn send_direct_msg<S>(
        &self,
        receiver: XOnlyPublicKey,
        msg: S,
        reply_to: Option<EventId>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder: EventBuilder = match self.signer().await? {
            ClientSigner::Keys(keys) => {
                EventBuilder::encrypted_direct_msg(&keys, receiver, msg, reply_to)?
            }
            #[cfg(all(feature = "nip07", target_arch = "wasm32"))]
            ClientSigner::NIP07(nip07) => {
                let content: String = nip07.nip04_encrypt(receiver, msg.into()).await?;
                EventBuilder::new(
                    Kind::EncryptedDirectMessage,
                    content,
                    [Tag::public_key(receiver)],
                )
            }
            #[cfg(feature = "nip46")]
            ClientSigner::NIP46(..) => {
                let req = nip46::Request::Nip04Encrypt {
                    public_key: receiver,
                    text: msg.into(),
                };
                let res: nip46::Response = self
                    .send_req_to_signer(req, self.opts.nip46_timeout)
                    .await?;
                if let nip46::Response::Nip04Encrypt(content) = res {
                    EventBuilder::new(
                        Kind::EncryptedDirectMessage,
                        content,
                        [Tag::public_key(receiver)],
                    )
                } else {
                    return Err(Error::ResponseNotMatchRequest);
                }
            }
        };

        self.send_event_builder(builder).await
    }

    /// Repost event
    pub async fn repost_event(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::repost(event_id, public_key);
        self.send_event_builder(builder).await
    }

    /// Delete event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/09.md>
    pub async fn delete_event<T>(&self, id: T) -> Result<EventId, Error>
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
    /// let event_id =
    ///     EventId::from_hex("3aded8d2194dc2fedb1d7b70480b43b6c4deb0a22dcdc9c471d1958485abcf21")
    ///         .unwrap();
    /// let public_key = XOnlyPublicKey::from_str(
    ///     "a8e76c3ace7829f9ee44cf9293309e21a1824bf1e57631d00685a1ed0b0bd8a2",
    /// )
    /// .unwrap();
    ///
    /// client.like(event_id, public_key).await.unwrap();
    /// # }
    /// ```
    pub async fn like(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::reaction(event_id, public_key, "+");
        self.send_event_builder(builder).await
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
    /// let event_id =
    ///     EventId::from_hex("3aded8d2194dc2fedb1d7b70480b43b6c4deb0a22dcdc9c471d1958485abcf21")
    ///         .unwrap();
    /// let public_key = XOnlyPublicKey::from_str(
    ///     "a8e76c3ace7829f9ee44cf9293309e21a1824bf1e57631d00685a1ed0b0bd8a2",
    /// )
    /// .unwrap();
    ///
    /// client.dislike(event_id, public_key).await.unwrap();
    /// # }
    /// ```
    pub async fn dislike(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::reaction(event_id, public_key, "-");
        self.send_event_builder(builder).await
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
    /// let event_id =
    ///     EventId::from_hex("3aded8d2194dc2fedb1d7b70480b43b6c4deb0a22dcdc9c471d1958485abcf21")
    ///         .unwrap();
    /// let public_key = XOnlyPublicKey::from_str(
    ///     "a8e76c3ace7829f9ee44cf9293309e21a1824bf1e57631d00685a1ed0b0bd8a2",
    /// )
    /// .unwrap();
    ///
    /// client.reaction(event_id, public_key, "🐻").await.unwrap();
    /// # }
    /// ```
    pub async fn reaction<S>(
        &self,
        event_id: EventId,
        public_key: XOnlyPublicKey,
        content: S,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::reaction(event_id, public_key, content);
        self.send_event_builder(builder).await
    }

    /// Create new channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn new_channel(&self, metadata: &Metadata) -> Result<EventId, Error> {
        let builder = EventBuilder::channel(metadata);
        self.send_event_builder(builder).await
    }

    /// Update channel metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn set_channel_metadata(
        &self,
        channel_id: EventId,
        relay_url: Option<Url>,
        metadata: &Metadata,
    ) -> Result<EventId, Error> {
        let builder = EventBuilder::channel_metadata(channel_id, relay_url, metadata);
        self.send_event_builder(builder).await
    }

    /// Send message to channel
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn send_channel_msg<S>(
        &self,
        channel_id: EventId,
        relay_url: Url,
        msg: S,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::channel_msg(channel_id, relay_url, msg);
        self.send_event_builder(builder).await
    }

    /// Hide channel message
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn hide_channel_msg<S>(
        &self,
        message_id: EventId,
        reason: Option<S>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::hide_channel_msg(message_id, reason);
        self.send_event_builder(builder).await
    }

    /// Mute channel user
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/28.md>
    pub async fn mute_channel_user<S>(
        &self,
        pubkey: XOnlyPublicKey,
        reason: Option<S>,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::mute_channel_user(pubkey, reason);
        self.send_event_builder(builder).await
    }

    /// Create an auth event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/42.md>
    pub async fn auth<S>(&self, challenge: S, relay: Url) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::auth(challenge, relay);
        self.send_event_builder(builder).await
    }

    /// Create zap receipt event
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[cfg(feature = "nip57")]
    pub async fn zap_receipt<S>(
        &self,
        bolt11: S,
        preimage: Option<S>,
        zap_request: Event,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::zap_receipt(bolt11, preimage, zap_request);
        self.send_event_builder(builder).await
    }

    /// Gift Wrap
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/59.md>
    #[cfg(feature = "nip59")]
    pub async fn gift_wrap(
        &self,
        receiver: XOnlyPublicKey,
        rumor: EventBuilder,
    ) -> Result<(), Error> {
        // Compose rumor
        let signer: ClientSigner = self.signer().await?;
        let public_key: XOnlyPublicKey = signer.public_key().await?;
        let rumor = rumor.to_unsigned_event(public_key);

        // Compose seal
        let content: String = self.nip44_encrypt(receiver, rumor.as_json()).await?;
        let seal: EventBuilder = EventBuilder::new(Kind::Seal, content, []);
        let seal: Event = self.sign_event_builder(seal).await?;

        // Compose gift wrap
        let gift_wrap: Event = EventBuilder::gift_wrap_from_seal(&receiver, &seal)?;

        // Send event
        self.send_event(gift_wrap).await?;

        Ok(())
    }

    /// Send GiftWrapper Sealed Direct message
    #[cfg(feature = "nip59")]
    pub async fn send_sealed_msg<S>(
        &self,
        receiver: XOnlyPublicKey,
        message: S,
    ) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let rumor: EventBuilder = EventBuilder::sealed_direct(receiver, message);
        self.gift_wrap(receiver, rumor).await
    }

    /// File metadata
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/94.md>
    pub async fn file_metadata<S>(
        &self,
        description: S,
        metadata: FileMetadata,
    ) -> Result<EventId, Error>
    where
        S: Into<String>,
    {
        let builder = EventBuilder::file_metadata(description, metadata);
        self.send_event_builder(builder).await
    }

    /// Negentropy reconciliation
    ///
    /// <https://github.com/hoytech/negentropy>
    pub async fn reconcile(&self, filter: Filter, opts: NegentropyOptions) -> Result<(), Error> {
        Ok(self.pool.reconcile(filter, opts).await?)
    }

    /// Negentropy reconciliation with items
    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<(), Error> {
        Ok(self.pool.reconcile_with_items(filter, items, opts).await?)
    }

    /// Handle notifications
    pub async fn handle_notifications<F, Fut>(&self, func: F) -> Result<(), Error>
    where
        F: Fn(RelayPoolNotification) -> Fut,
        Fut: Future<Output = Result<bool>>,
    {
        let mut notifications = self.notifications();
        while let Ok(notification) = notifications.recv().await {
            let stop: bool = RelayPoolNotification::Stop == notification;
            let shutdown: bool = RelayPoolNotification::Shutdown == notification;
            let exit: bool = func(notification)
                .await
                .map_err(|e| Error::Handler(e.to_string()))?;
            if exit || stop || shutdown {
                break;
            }
        }
        Ok(())
    }
}
