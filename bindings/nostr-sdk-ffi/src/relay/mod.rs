// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::{ClientMessage, Event, EventId, Filter, RelayInformationDocument};
use nostr_sdk::database::DynNostrDatabase;
use nostr_sdk::{block_on, pool, FilterOptions, SubscriptionId, Url};
use uniffi::Object;

pub mod limits;
pub mod options;
pub mod stats;
pub mod status;

pub use self::limits::RelayLimits;
use self::options::NegentropyOptions;
pub use self::options::{RelayOptions, RelaySendOptions, SubscribeOptions};
pub use self::stats::RelayConnectionStats;
pub use self::status::RelayStatus;
use crate::error::Result;
use crate::negentropy::NegentropyItem;
use crate::NostrDatabase;

#[derive(Object)]
pub struct Relay {
    inner: pool::Relay,
}

impl From<pool::Relay> for Relay {
    fn from(inner: pool::Relay) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Relay {
    /// Create new `Relay` with **default** `options` and `in-memory database`
    #[uniffi::constructor]
    pub fn new(url: &str) -> Result<Self> {
        let url: Url = Url::parse(url)?;
        Ok(Self {
            inner: nostr_sdk::Relay::new(url),
        })
    }

    /// Create new `Relay` with default `in-memory database` and custom `options`
    #[uniffi::constructor]
    pub fn with_opts(url: &str, opts: &RelayOptions) -> Result<Self> {
        let url: Url = Url::parse(url)?;
        let opts = opts.deref().clone();
        Ok(Self {
            inner: nostr_sdk::Relay::with_opts(url, opts),
        })
    }

    /// Create new `Relay` with **custom** `options` and/or `database`
    #[uniffi::constructor]
    pub fn custom(url: String, database: &NostrDatabase, opts: &RelayOptions) -> Result<Self> {
        let url: Url = Url::parse(&url)?;
        let database: Arc<DynNostrDatabase> = database.into();
        let opts = opts.deref().clone();
        Ok(Self {
            inner: nostr_sdk::Relay::custom(url, database, opts),
        })
    }

    /// Get relay url
    pub fn url(&self) -> String {
        self.inner.url().to_string()
    }

    /// Get proxy
    pub fn proxy(&self) -> Option<String> {
        self.inner.proxy().map(|p| p.to_string())
    }

    /// Get relay status
    pub fn status(&self) -> RelayStatus {
        block_on(async move { self.inner.status().await.into() })
    }

    /* /// Get Relay Service Flags
    pub fn flags(&self) -> AtomicRelayServiceFlags {
        self.inner.flags()
    } */

    /// Check if `Relay` is connected
    pub fn is_connected(&self) -> bool {
        block_on(async move { self.inner.is_connected().await })
    }

    pub fn document(&self) -> Arc<RelayInformationDocument> {
        block_on(async move { Arc::new(self.inner.document().await.into()) })
    }

    pub fn subscriptions(&self) -> HashMap<String, Vec<Arc<Filter>>> {
        block_on(async move {
            self.inner
                .subscriptions()
                .await
                .into_iter()
                .map(|(id, filters)| {
                    (
                        id.to_string(),
                        filters.into_iter().map(|f| Arc::new(f.into())).collect(),
                    )
                })
                .collect()
        })
    }

    /// Get filters by subscription ID
    pub fn subscription(&self, id: String) -> Option<Vec<Arc<Filter>>> {
        block_on(async move {
            let id = SubscriptionId::new(id);
            self.inner
                .subscription(&id)
                .await
                .map(|f| f.into_iter().map(|f| Arc::new(f.into())).collect())
        })
    }

    pub fn opts(&self) -> RelayOptions {
        self.inner.opts().into()
    }

    pub fn stats(&self) -> Arc<RelayConnectionStats> {
        Arc::new(self.inner.stats().into())
    }

    /// Get number of messages in queue
    pub fn queue(&self) -> u64 {
        self.inner.queue() as u64
    }

    // TODO: add notifications

    /// Connect to relay and keep alive connection
    pub fn connect(&self, connection_timeout: Option<Duration>) {
        block_on(async move { self.inner.connect(connection_timeout).await })
    }

    /// Disconnect from relay and set status to 'Stopped'
    pub fn stop(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.stop().await?) })
    }

    /// Disconnect from relay and set status to 'Terminated'
    pub fn terminate(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.terminate().await?) })
    }

    /// Send msg to relay
    pub fn send_msg(&self, msg: Arc<ClientMessage>, opts: Arc<RelaySendOptions>) -> Result<()> {
        block_on(async move {
            Ok(self
                .inner
                .send_msg(msg.as_ref().deref().clone(), **opts)
                .await?)
        })
    }

    /// Send multiple `ClientMessage` at once
    pub fn batch_msg(&self, msgs: Vec<Arc<ClientMessage>>, opts: &RelaySendOptions) -> Result<()> {
        let msgs = msgs
            .into_iter()
            .map(|msg| msg.as_ref().deref().clone())
            .collect();
        block_on(async move { Ok(self.inner.batch_msg(msgs, **opts).await?) })
    }

    /// Send event and wait for `OK` relay msg
    pub fn send_event(&self, event: &Event, opts: &RelaySendOptions) -> Result<Arc<EventId>> {
        block_on(async move {
            Ok(Arc::new(
                self.inner
                    .send_event(event.deref().clone(), **opts)
                    .await?
                    .into(),
            ))
        })
    }

    /// Send multiple `Event` at once
    pub fn batch_event(&self, events: Vec<Arc<Event>>, opts: &RelaySendOptions) -> Result<()> {
        let events = events
            .into_iter()
            .map(|e| e.as_ref().deref().clone())
            .collect();
        block_on(async move { Ok(self.inner.batch_event(events, **opts).await?) })
    }

    /// Subscribe to filters
    ///
    /// Internally generate a new random subscription ID. Check `subscribe_with_id` method to use a custom subscription ID.
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub fn subscribe(&self, filters: Vec<Arc<Filter>>, opts: &SubscribeOptions) -> Result<String> {
        block_on(async move {
            Ok(self
                .inner
                .subscribe(
                    filters
                        .into_iter()
                        .map(|f| f.as_ref().deref().clone())
                        .collect(),
                    **opts,
                )
                .await?
                .to_string())
        })
    }

    /// Subscribe with custom subscription ID
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub fn subscribe_with_id(
        &self,
        id: String,
        filters: Vec<Arc<Filter>>,
        opts: &SubscribeOptions,
    ) -> Result<()> {
        block_on(async move {
            Ok(self
                .inner
                .subscribe_with_id(
                    SubscriptionId::new(id),
                    filters
                        .into_iter()
                        .map(|f| f.as_ref().deref().clone())
                        .collect(),
                    **opts,
                )
                .await?)
        })
    }

    /// Unsubscribe
    pub fn unsubscribe(&self, id: String, opts: Arc<RelaySendOptions>) -> Result<()> {
        block_on(async move {
            Ok(self
                .inner
                .unsubscribe(SubscriptionId::new(id), **opts)
                .await?)
        })
    }

    /// Unsubscribe from all subscriptions
    pub fn unsubscribe_all(&self, opts: Arc<RelaySendOptions>) -> Result<()> {
        block_on(async move { Ok(self.inner.unsubscribe_all(**opts).await?) })
    }

    /// Get events of filters
    ///
    /// Get events from local database and relay
    pub fn get_events_of(
        &self,
        filters: Vec<Arc<Filter>>,
        timeout: Duration,
    ) -> Result<Vec<Arc<Event>>> {
        block_on(async move {
            let filters = filters
                .into_iter()
                .map(|f| f.as_ref().deref().clone())
                .collect();
            Ok(self
                .inner
                .get_events_of(filters, timeout, FilterOptions::ExitOnEOSE)
                .await?
                .into_iter()
                .map(|e| Arc::new(e.into()))
                .collect())
        })
    }

    /// Count events of filters
    pub fn count_events_of(&self, filters: Vec<Arc<Filter>>, timeout: Duration) -> Result<u64> {
        block_on(async move {
            let filters = filters
                .into_iter()
                .map(|f| f.as_ref().deref().clone())
                .collect();
            Ok(self.inner.count_events_of(filters, timeout).await? as u64)
        })
    }

    /// Negentropy reconciliation
    ///
    /// Use events stored in database
    pub fn reconcile(&self, filter: &Filter, opts: &NegentropyOptions) -> Result<()> {
        block_on(async move { Ok(self.inner.reconcile(filter.deref().clone(), **opts).await?) })
    }

    /// Negentropy reconciliation with custom items
    pub fn reconcile_with_items(
        &self,
        filter: &Filter,
        items: Vec<NegentropyItem>,
        opts: &NegentropyOptions,
    ) -> Result<()> {
        block_on(async move {
            let items = items
                .into_iter()
                .map(|item| (**item.id, **item.timestamp))
                .collect();
            Ok(self
                .inner
                .reconcile_with_items(filter.deref().clone(), items, **opts)
                .await?)
        })
    }

    /// Check if relay support negentropy protocol
    pub fn support_negentropy(&self) -> Result<bool> {
        block_on(async move { Ok(self.inner.support_negentropy().await?) })
    }
}
