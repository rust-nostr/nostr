// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::{ClientMessage, Event, Filter};
use nostr_sdk::database::DynNostrDatabase;
use nostr_sdk::{RelayPoolOptions, SubscriptionId};
use uniffi::Object;

pub mod result;

use self::result::{Output, SendEventOutput, SubscribeOutput};
use crate::error::Result;
use crate::negentropy::NegentropyItem;
use crate::pool::result::ReconciliationOutput;
use crate::relay::options::{FilterOptions, NegentropyOptions};
use crate::relay::{RelayFiltering, RelayOptions, RelaySendOptions, SubscribeOptions};
use crate::{HandleNotification, NostrDatabase, Relay};

#[derive(Object)]
pub struct RelayPool {
    inner: nostr_sdk::RelayPool,
}

impl From<nostr_sdk::RelayPool> for RelayPool {
    fn from(inner: nostr_sdk::RelayPool) -> Self {
        Self { inner }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RelayPool {
    /// Create new `RelayPool` with `in-memory` database
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr_sdk::RelayPool::new(RelayPoolOptions::default()),
        }
    }

    /// Create new `RelayPool` with `custom` database
    #[uniffi::constructor]
    pub fn with_database(database: &NostrDatabase) -> Self {
        let database: Arc<DynNostrDatabase> = database.into();
        Self {
            inner: nostr_sdk::RelayPool::with_database(RelayPoolOptions::default(), database),
        }
    }

    /// Completely shutdown pool
    pub async fn shutdown(&self) -> Result<()> {
        Ok(self.inner.shutdown().await?)
    }

    /// Get database
    pub fn database(&self) -> Arc<NostrDatabase> {
        Arc::new(self.inner.database().into())
    }

    /// Get relay filtering
    pub fn filtering(&self) -> RelayFiltering {
        self.inner.filtering().into()
    }

    /// Get relays with `READ` or `WRITE` flags
    pub async fn relays(&self) -> HashMap<String, Arc<Relay>> {
        self.inner
            .relays()
            .await
            .into_iter()
            .map(|(u, r)| (u.to_string(), Arc::new(r.into())))
            .collect()
    }

    /// Get relay
    pub async fn relay(&self, url: String) -> Result<Arc<Relay>> {
        Ok(Arc::new(self.inner.relay(url).await?.into()))
    }

    pub async fn add_relay(&self, url: String, opts: &RelayOptions) -> Result<bool> {
        Ok(self.inner.add_relay(url, opts.deref().clone()).await?)
    }

    /// Remove and disconnect relay
    ///
    /// If the relay has `INBOX` or `OUTBOX` flags, it will not be removed from the pool and its
    /// flags will be updated (remove `READ`, `WRITE` and `DISCOVERY` flags).
    pub async fn remove_relay(&self, url: &str) -> Result<()> {
        Ok(self.inner.remove_relay(url).await?)
    }

    /// Force remove and disconnect relay
    ///
    /// Note: this method will remove the relay, also if it's in use for the gossip model or other service!
    pub async fn force_remove_relay(&self, url: &str) -> Result<()> {
        Ok(self.inner.force_remove_relay(url).await?)
    }

    /// Connect to all added relays and keep connection alive
    pub async fn connect(&self, connection_timeout: Option<Duration>) {
        self.inner.connect(connection_timeout).await
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) -> Result<()> {
        Ok(self.inner.disconnect().await?)
    }

    /// Connect to relay
    pub async fn connect_relay(
        &self,
        url: String,
        connection_timeout: Option<Duration>,
    ) -> Result<()> {
        Ok(self.inner.connect_relay(url, connection_timeout).await?)
    }

    /// Get subscriptions
    pub async fn subscriptions(&self) -> HashMap<String, Vec<Arc<Filter>>> {
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
    }

    /// Get filters by subscription ID
    pub async fn subscription(&self, id: String) -> Option<Vec<Arc<Filter>>> {
        let id = SubscriptionId::new(id);
        self.inner
            .subscription(&id)
            .await
            .map(|f| f.into_iter().map(|f| Arc::new(f.into())).collect())
    }

    /// Send client message to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn send_msg_to(
        &self,
        urls: Vec<String>,
        msg: Arc<ClientMessage>,
        opts: Arc<RelaySendOptions>,
    ) -> Result<Output> {
        Ok(self
            .inner
            .send_msg_to(urls, msg.as_ref().deref().clone(), **opts)
            .await?
            .into())
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub async fn batch_msg_to(
        &self,
        urls: Vec<String>,
        msgs: Vec<Arc<ClientMessage>>,
        opts: &RelaySendOptions,
    ) -> Result<Output> {
        let msgs = msgs
            .into_iter()
            .map(|msg| msg.as_ref().deref().clone())
            .collect();
        Ok(self.inner.batch_msg_to(urls, msgs, **opts).await?.into())
    }

    /// Send event to all relays with `WRITE` flag
    pub async fn send_event(
        &self,
        event: &Event,
        opts: &RelaySendOptions,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event(event.deref().clone(), **opts)
            .await?
            .into())
    }

    /// Send multiple events at once to all relays with `WRITE` flag
    pub async fn batch_event(
        &self,
        events: Vec<Arc<Event>>,
        opts: &RelaySendOptions,
    ) -> Result<Output> {
        let events = events
            .into_iter()
            .map(|e| e.as_ref().deref().clone())
            .collect();
        Ok(self.inner.batch_event(events, **opts).await?.into())
    }

    /// Send event to specific relays
    pub async fn send_event_to(
        &self,
        urls: Vec<String>,
        event: &Event,
        opts: &RelaySendOptions,
    ) -> Result<SendEventOutput> {
        Ok(self
            .inner
            .send_event_to(urls, event.deref().clone(), **opts)
            .await?
            .into())
    }

    /// Send multiple events at once to specific relays
    pub async fn batch_event_to(
        &self,
        urls: Vec<String>,
        events: Vec<Arc<Event>>,
        opts: &RelaySendOptions,
    ) -> Result<Output> {
        let events = events
            .into_iter()
            .map(|e| e.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .batch_event_to(urls, events, **opts)
            .await?
            .into())
    }

    /// Subscribe to filters to relays with `READ` flag.
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub async fn subscribe(
        &self,
        filters: Vec<Arc<Filter>>,
        opts: &SubscribeOptions,
    ) -> Result<SubscribeOutput> {
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
            .into())
    }

    /// Subscribe with custom subscription ID to relays with `READ` flag.
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub async fn subscribe_with_id(
        &self,
        id: String,
        filters: Vec<Arc<Filter>>,
        opts: &SubscribeOptions,
    ) -> Result<Output> {
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
            .await?
            .into())
    }

    /// Subscribe to filters to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    pub async fn subscribe_to(
        &self,
        urls: Vec<String>,
        filters: Vec<Arc<Filter>>,
        opts: &SubscribeOptions,
    ) -> Result<SubscribeOutput> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self.inner.subscribe_to(urls, filters, **opts).await?.into())
    }

    /// Subscribe to filters with custom subscription ID to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    pub async fn subscribe_with_id_to(
        &self,
        urls: Vec<String>,
        id: String,
        filters: Vec<Arc<Filter>>,
        opts: &SubscribeOptions,
    ) -> Result<Output> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .subscribe_with_id_to(urls, SubscriptionId::new(id), filters, **opts)
            .await?
            .into())
    }

    /// Unsubscribe
    pub async fn unsubscribe(&self, id: String, opts: Arc<RelaySendOptions>) {
        self.inner
            .unsubscribe(SubscriptionId::new(id), **opts)
            .await
    }

    /// Unsubscribe from all subscriptions
    pub async fn unsubscribe_all(&self, opts: Arc<RelaySendOptions>) {
        self.inner.unsubscribe_all(**opts).await
    }

    /// Get events of filters
    pub async fn get_events_of(
        &self,
        filters: Vec<Arc<Filter>>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Arc<Event>>> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .get_events_of(filters, timeout, opts.into())
            .await?
            .into_iter()
            .map(|e| Arc::new(e.into()))
            .collect())
    }

    /// Get events of filters from **specific relays**
    pub async fn get_events_from(
        &self,
        urls: Vec<String>,
        filters: Vec<Arc<Filter>>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Arc<Event>>> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        Ok(self
            .inner
            .get_events_from(urls, filters, timeout, opts.into())
            .await?
            .into_iter()
            .map(|e| Arc::new(e.into()))
            .collect())
    }

    /// Negentropy reconciliation
    ///
    /// Use events stored in database
    pub async fn reconcile(
        &self,
        filter: &Filter,
        opts: &NegentropyOptions,
    ) -> Result<ReconciliationOutput> {
        Ok(self
            .inner
            .reconcile(filter.deref().clone(), **opts)
            .await?
            .into())
    }

    /// Negentropy reconciliation with custom items
    pub async fn reconcile_with_items(
        &self,
        filter: &Filter,
        items: Vec<NegentropyItem>,
        opts: &NegentropyOptions,
    ) -> Result<ReconciliationOutput> {
        let items = items
            .into_iter()
            .map(|item| (**item.id, **item.timestamp))
            .collect();
        Ok(self
            .inner
            .reconcile_with_items(filter.deref().clone(), items, **opts)
            .await?
            .into())
    }

    /// Handle relay pool notifications
    pub async fn handle_notifications(&self, handler: Arc<dyn HandleNotification>) -> Result<()> {
        Ok(self
            .inner
            .handle_notifications(|notification| async {
                match notification {
                    nostr_sdk::RelayPoolNotification::Message { relay_url, message } => {
                        handler
                            .handle_msg(relay_url.to_string(), Arc::new(message.into()))
                            .await;
                    }
                    nostr_sdk::RelayPoolNotification::Event {
                        relay_url,
                        subscription_id,
                        event,
                    } => {
                        handler
                            .handle(
                                relay_url.to_string(),
                                subscription_id.to_string(),
                                Arc::new((*event).into()),
                            )
                            .await;
                    }
                    _ => (),
                }
                Ok(false)
            })
            .await?)
    }
}
