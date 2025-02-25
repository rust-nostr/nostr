// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_sdk::{pool, RelayUrl, SubscriptionId};
use uniffi::{Object, Record};

pub mod limits;
pub mod options;
pub mod stats;
pub mod status;

pub use self::limits::RelayLimits;
use self::options::SyncOptions;
pub use self::options::{ConnectionMode, RelayOptions, ReqExitPolicy, SubscribeOptions};
pub use self::stats::RelayConnectionStats;
pub use self::status::RelayStatus;
use crate::database::events::Events;
use crate::database::NostrDatabase;
use crate::error::Result;
use crate::negentropy::NegentropyItem;
use crate::protocol::event::{Event, EventId};
use crate::protocol::filter::Filter;
use crate::protocol::message::ClientMessage;
use crate::protocol::nips::nip11::RelayInformationDocument;

#[derive(Record)]
pub struct ReconciliationSendFailureItem {
    pub id: Arc<EventId>,
    pub error: String,
}

/// Reconciliation output
#[derive(Record)]
pub struct Reconciliation {
    /// The IDs that were stored locally
    pub local: Vec<Arc<EventId>>,
    /// The IDs that were missing locally (stored on relay)
    pub remote: Vec<Arc<EventId>>,
    /// Events that are **successfully** sent to relays during reconciliation
    pub sent: Vec<Arc<EventId>>,
    /// Event that are **successfully** received from relay
    pub received: Vec<Arc<EventId>>,

    pub send_failures: HashMap<String, Vec<ReconciliationSendFailureItem>>,
}

impl From<pool::Reconciliation> for Reconciliation {
    fn from(value: pool::Reconciliation) -> Self {
        Self {
            local: value
                .local
                .into_iter()
                .map(|e| Arc::new(e.into()))
                .collect(),
            remote: value
                .remote
                .into_iter()
                .map(|e| Arc::new(e.into()))
                .collect(),
            sent: value.sent.into_iter().map(|e| Arc::new(e.into())).collect(),
            received: value
                .received
                .into_iter()
                .map(|e| Arc::new(e.into()))
                .collect(),
            send_failures: value
                .send_failures
                .into_iter()
                .map(|(url, map)| {
                    (
                        url.to_string(),
                        map.into_iter()
                            .map(|(id, e)| ReconciliationSendFailureItem {
                                id: Arc::new(id.into()),
                                error: e,
                            })
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Object)]
pub struct Relay {
    inner: pool::Relay,
}

impl From<pool::Relay> for Relay {
    fn from(inner: pool::Relay) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl Relay {
    /// Create new `Relay` with **default** `options` and `in-memory database`
    #[uniffi::constructor]
    pub fn new(url: &str) -> Result<Self> {
        let url: RelayUrl = RelayUrl::parse(url)?;
        Ok(Self {
            inner: nostr_sdk::Relay::new(url),
        })
    }

    /// Create new `Relay` with default `in-memory database` and custom `options`
    #[uniffi::constructor]
    pub fn with_opts(url: &str, opts: &RelayOptions) -> Result<Self> {
        let url: RelayUrl = RelayUrl::parse(url)?;
        let opts = opts.deref().clone();
        Ok(Self {
            inner: nostr_sdk::Relay::with_opts(url, opts),
        })
    }

    /// Create new `Relay` with **custom** `database` and/or `options`
    #[uniffi::constructor]
    pub fn custom(url: &str, database: &NostrDatabase, opts: &RelayOptions) -> Result<Self> {
        let url: RelayUrl = RelayUrl::parse(url)?;
        Ok(Self {
            inner: nostr_sdk::Relay::custom(url, database.deref().clone(), opts.deref().clone()),
        })
    }

    /// Get relay url
    pub fn url(&self) -> String {
        self.inner.url().to_string()
    }

    /// Get connection mode
    pub fn connection_mode(&self) -> ConnectionMode {
        self.inner.connection_mode().clone().into()
    }

    /// Get status
    pub fn status(&self) -> RelayStatus {
        self.inner.status().into()
    }

    /* /// Get Relay Service Flags
    pub fn flags(&self) -> AtomicRelayServiceFlags {
        self.inner.flags()
    } */

    /// Check if `Relay` is connected
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    pub async fn document(&self) -> RelayInformationDocument {
        self.inner.document().await.into()
    }

    pub async fn subscriptions(&self) -> HashMap<String, Arc<Filter>> {
        self.inner
            .subscriptions()
            .await
            .into_iter()
            .map(|(id, f)| (id.to_string(), Arc::new(f.into())))
            .collect()
    }

    /// Get filters by subscription ID
    pub async fn subscription(&self, id: String) -> Option<Arc<Filter>> {
        let id = SubscriptionId::new(id);
        self.inner
            .subscription(&id)
            .await
            .map(|f| Arc::new(f.into()))
    }

    pub fn opts(&self) -> RelayOptions {
        self.inner.opts().clone().into()
    }

    pub fn stats(&self) -> RelayConnectionStats {
        self.inner.stats().clone().into()
    }

    /// Get number of messages in queue
    pub fn queue(&self) -> u64 {
        self.inner.queue() as u64
    }

    // TODO: add notifications

    /// Connect to relay
    ///
    /// This method returns immediately and doesn't provide any information on if the connection was successful or not.
    pub fn connect(&self) {
        self.inner.connect()
    }

    /// Try to connect to relay
    ///
    /// This method returns an error if the connection fails.
    /// If the connection fails,
    /// a task will continue to retry in the background (unless configured differently in `RelayOptions`.
    pub async fn try_connect(&self, timeout: Duration) -> Result<()> {
        Ok(self.inner.try_connect(timeout).await?)
    }

    /// Disconnect from relay and set status to 'Terminated'
    pub fn disconnect(&self) {
        self.inner.disconnect()
    }

    /// Send msg to relay
    pub fn send_msg(&self, msg: &ClientMessage) -> Result<()> {
        Ok(self.inner.send_msg(msg.deref().clone())?)
    }

    /// Send multiple `ClientMessage` at once
    pub fn batch_msg(&self, msgs: Vec<Arc<ClientMessage>>) -> Result<()> {
        let msgs = msgs
            .into_iter()
            .map(|msg| msg.as_ref().deref().clone())
            .collect();
        Ok(self.inner.batch_msg(msgs)?)
    }

    /// Send event and wait for `OK` relay msg
    pub async fn send_event(&self, event: &Event) -> Result<Arc<EventId>> {
        Ok(Arc::new(self.inner.send_event(event.deref()).await?.into()))
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
    pub async fn subscribe(&self, filter: &Filter, opts: &SubscribeOptions) -> Result<String> {
        Ok(self
            .inner
            .subscribe(filter.deref().clone(), **opts)
            .await?
            .to_string())
    }

    /// Subscribe with custom subscription ID
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub async fn subscribe_with_id(
        &self,
        id: String,
        filter: &Filter,
        opts: &SubscribeOptions,
    ) -> Result<()> {
        Ok(self
            .inner
            .subscribe_with_id(SubscriptionId::new(id), filter.deref().clone(), **opts)
            .await?)
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self, id: String) -> Result<()> {
        Ok(self.inner.unsubscribe(&SubscriptionId::new(id)).await?)
    }

    /// Unsubscribe from all subscriptions
    pub async fn unsubscribe_all(&self) -> Result<()> {
        Ok(self.inner.unsubscribe_all().await?)
    }

    /// Fetch events
    pub async fn fetch_events(
        &self,
        filter: &Filter,
        timeout: Duration,
        policy: ReqExitPolicy,
    ) -> Result<Events> {
        Ok(self
            .inner
            .fetch_events(filter.deref().clone(), timeout, policy.into())
            .await?
            .into())
    }

    /// Count events
    pub async fn count_events(&self, filter: &Filter, timeout: Duration) -> Result<u64> {
        Ok(self
            .inner
            .count_events(filter.deref().clone(), timeout)
            .await? as u64)
    }

    /// Sync events with relays (negentropy reconciliation)
    pub async fn sync(&self, filter: &Filter, opts: &SyncOptions) -> Result<Reconciliation> {
        Ok(self
            .inner
            .sync(filter.deref().clone(), opts.deref())
            .await?
            .into())
    }

    /// Sync events with relays (negentropy reconciliation)
    pub async fn sync_with_items(
        &self,
        filter: &Filter,
        items: Vec<NegentropyItem>,
        opts: &SyncOptions,
    ) -> Result<Reconciliation> {
        let items = items
            .into_iter()
            .map(|item| (**item.id, **item.timestamp))
            .collect();
        Ok(self
            .inner
            .sync_with_items(filter.deref().clone(), items, opts.deref())
            .await?
            .into())
    }
}
