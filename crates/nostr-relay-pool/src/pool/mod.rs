// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay Pool

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_utility::thread;
use nostr::{ClientMessage, Event, EventId, Filter, RelayMessage, Timestamp, TryIntoUrl, Url};
use nostr_database::{DynNostrDatabase, IntoNostrDatabase, MemoryDatabase};
use tokio::sync::broadcast;

pub mod options;
mod internal;

pub use self::internal::Error;
use self::internal::InternalRelayPool;
pub use self::options::RelayPoolOptions;
use crate::relay::options::{FilterOptions, NegentropyOptions, RelayOptions, RelaySendOptions};
use crate::relay::{Relay, RelayStatus};
use crate::util::SaturatingUsize;

/// Relay Pool Notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayPoolNotification {
    /// Received an [`Event`]. Does not include events sent by this client.
    Event {
        /// Relay url
        relay_url: Url,
        /// Event
        event: Event,
    },
    /// Received a [`RelayMessage`]. Includes messages wrapping events that were sent by this client.
    Message {
        /// Relay url
        relay_url: Url,
        /// Relay Message
        message: RelayMessage,
    },
    /// Relay status changed
    RelayStatus {
        /// Relay url
        relay_url: Url,
        /// Relay Status
        status: RelayStatus,
    },
    /// Stop
    Stop,
    /// Shutdown
    Shutdown,
}

/// Relay Pool
#[derive(Debug)]
pub struct RelayPool {
    internal: InternalRelayPool,
    shutdown: Arc<AtomicBool>,
    ref_counter: Arc<AtomicUsize>,
}

impl Default for RelayPool {
    fn default() -> Self {
        Self::new(RelayPoolOptions::default())
    }
}

impl Clone for RelayPool {
    fn clone(&self) -> Self {
        // Increase counter
        let new_ref_counter: usize = self.ref_counter.saturating_increment(Ordering::SeqCst);
        tracing::debug!("Relay Pool cloned: ref counter increased to {new_ref_counter}");

        // Clone
        Self {
            internal: self.internal.clone(),
            shutdown: self.shutdown.clone(),
            ref_counter: self.ref_counter.clone(),
        }
    }
}

impl Drop for RelayPool {
    fn drop(&mut self) {
        // Check if already shutdown
        if self.shutdown.load(Ordering::SeqCst) {
            tracing::debug!("Relay Pool already shutdown");
        } else {
            // Decrease counter
            let new_ref_counter: usize = self.ref_counter.saturating_decrement(Ordering::SeqCst);
            tracing::debug!("Relay Pool dropped: ref counter decreased to {new_ref_counter}");

            // Check if it's time for shutdown
            if new_ref_counter == 0 {
                tracing::debug!("Shutting down Relay Pool...");

                // Mark as shutdown
                self.shutdown.store(true, Ordering::SeqCst);

                // Clone internal pool and shutdown
                let pool: InternalRelayPool = self.internal.clone();
                let _ = thread::spawn(async move {
                    if let Err(e) = pool.shutdown().await {
                        tracing::error!("Impossible to shutdown Relay Pool: {e}");
                    }
                });

                tracing::info!("Relay Pool shutdown.");
            }
        }
    }
}

impl RelayPool {
    /// Create new `RelayPool`
    pub fn new(opts: RelayPoolOptions) -> Self {
        Self::with_database(opts, Arc::new(MemoryDatabase::default()))
    }

    /// New with database
    pub fn with_database<D>(opts: RelayPoolOptions, database: D) -> Self
    where
        D: IntoNostrDatabase,
    {
        Self {
            internal: InternalRelayPool::with_database(opts, database),
            shutdown: Arc::new(AtomicBool::new(false)),
            ref_counter: Arc::new(AtomicUsize::new(1)),
        }
    }

    /// Stop
    ///
    /// Call `connect` to re-start relays connections
    pub async fn stop(&self) -> Result<(), Error> {
        self.internal.stop().await
    }

    /// Completely shutdown pool
    pub async fn shutdown(self) -> Result<(), Error> {
        self.internal.shutdown().await
    }

    /// Get new **pool** notification listener
    pub fn notifications(&self) -> broadcast::Receiver<RelayPoolNotification> {
        self.internal.notifications()
    }

    /// Get database
    pub fn database(&self) -> Arc<DynNostrDatabase> {
        self.internal.database()
    }

    /// Get relays
    pub async fn relays(&self) -> HashMap<Url, Relay> {
        self.internal.relays().await
    }

    /// Get [`Relay`]
    pub async fn relay<U>(&self, url: U) -> Result<Relay, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.internal.relay(url).await
    }

    /// Get subscription filters
    pub async fn subscription_filters(&self) -> Vec<Filter> {
        self.internal.subscription_filters().await
    }

    /// Add new relay
    pub async fn add_relay<U>(&self, url: U, opts: RelayOptions) -> Result<bool, Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.internal.add_relay(url, opts).await
    }

    /// Disconnect and remove relay
    pub async fn remove_relay<U>(&self, url: U) -> Result<(), Error>
    where
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.internal.remove_relay(url).await
    }

    /// Disconnect and remove all relays
    pub async fn remove_all_relays(&self) -> Result<(), Error> {
        self.internal.remove_all_relays().await
    }

    /// Send client message
    pub async fn send_msg(&self, msg: ClientMessage, opts: RelaySendOptions) -> Result<(), Error> {
        let relays = self.relays().await;
        self.send_msg_to(relays.into_keys(), msg, opts).await
    }

    /// Send multiple client messages at once
    pub async fn batch_msg(
        &self,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        let relays = self.relays().await;
        self.batch_msg_to(relays.into_keys(), msgs, opts).await
    }

    /// Send client message to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn send_msg_to<I, U>(
        &self,
        urls: I,
        msg: ClientMessage,
        opts: RelaySendOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.batch_msg_to(urls, vec![msg], opts).await
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn batch_msg_to<I, U>(
        &self,
        urls: I,
        msgs: Vec<ClientMessage>,
        opts: RelaySendOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.internal.batch_msg_to(urls, msgs, opts).await
    }

    /// Send event and wait for `OK` relay msg
    pub async fn send_event(&self, event: Event, opts: RelaySendOptions) -> Result<EventId, Error> {
        let relays: HashMap<Url, Relay> = self.relays().await;
        self.send_event_to(relays.into_keys(), event, opts).await
    }

    /// Send multiple [`Event`] at once
    pub async fn batch_event(
        &self,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error> {
        let relays = self.relays().await;
        self.batch_event_to(relays.into_keys(), events, opts).await
    }

    /// Send event to a specific relays
    pub async fn send_event_to<I, U>(
        &self,
        urls: I,
        event: Event,
        opts: RelaySendOptions,
    ) -> Result<EventId, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let event_id: EventId = event.id;
        self.batch_event_to(urls, vec![event], opts).await?;
        Ok(event_id)
    }

    /// Send event to a specific relays
    pub async fn batch_event_to<I, U>(
        &self,
        urls: I,
        events: Vec<Event>,
        opts: RelaySendOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.internal.batch_event_to(urls, events, opts).await
    }

    /// Subscribe to filters
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Pool`
    pub async fn subscribe(&self, filters: Vec<Filter>, opts: RelaySendOptions) {
        self.internal.subscribe(filters, opts).await
    }

    /// Unsubscribe from filters
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Pool`
    pub async fn unsubscribe(&self, opts: RelaySendOptions) {
        self.internal.unsubscribe(opts).await
    }

    /// Get events of filters
    ///
    /// Get events both from **local database** and **relays**
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        let relays = self.relays().await;
        self.get_events_from(relays.into_keys(), filters, timeout, opts)
            .await
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
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Event>, Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        self.internal.get_events_from(urls, filters, timeout, opts).await
    }

    /// Request events of filter.
    ///
    /// If the events aren't already stored in the database, will be sent to notification listener
    /// until the EOSE "end of stored events" message is received from the relay.
    pub async fn req_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) {
        let relays = self.relays().await;
        for relay in relays.values() {
            relay.req_events_of(filters.clone(), timeout, opts);
        }
    }

    /// Request events of filter from specific relays.
    ///
    /// If the events aren't already stored in the database, will be sent to notification listener
    /// until the EOSE "end of stored events" message is received from the relay.
    pub async fn req_events_from<I, U>(
        &self,
        urls: I,
        filters: Vec<Filter>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = U>,
        U: TryIntoUrl,
        Error: From<<U as TryIntoUrl>::Err>,
    {
        let urls: HashSet<Url> = urls
            .into_iter()
            .map(|u| u.try_into_url())
            .collect::<Result<_, _>>()?;
        let relays: HashMap<Url, Relay> = self.relays().await;
        for (_, relay) in relays.into_iter().filter(|(url, ..)| urls.contains(url)) {
            relay.req_events_of(filters.clone(), timeout, opts);
        }
        Ok(())
    }

    /// Connect to all added relays and keep connection alive
    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        self.internal.connect(connection_timeout).await
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) -> Result<(), Error> {
        self.internal.disconnect().await
    }

    /// Connect to relay
    ///
    /// Internal Subscription ID set to `InternalSubscriptionId::Pool`
    pub async fn connect_relay(&self, relay: &Relay, connection_timeout: Option<Duration>) {
        self.internal.connect_relay(relay, connection_timeout).await
    }

    /// Negentropy reconciliation
    pub async fn reconcile(&self, filter: Filter, opts: NegentropyOptions) -> Result<(), Error> {
        self.internal.reconcile(filter, opts).await
    }

    /// Negentropy reconciliation with custom items
    pub async fn reconcile_with_items(
        &self,
        filter: Filter,
        items: Vec<(EventId, Timestamp)>,
        opts: NegentropyOptions,
    ) -> Result<(), Error> {
        self.internal.reconcile_with_items(filter, items, opts).await
    }
}
