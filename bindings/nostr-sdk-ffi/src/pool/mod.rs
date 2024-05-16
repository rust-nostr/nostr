// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use async_utility::thread;
use nostr_ffi::{ClientMessage, Event, EventId, Filter};
use nostr_sdk::database::DynNostrDatabase;
use nostr_sdk::{block_on, spawn_blocking, RelayPoolOptions, SubscriptionId};
use uniffi::Object;

use crate::error::Result;
use crate::negentropy::NegentropyItem;
use crate::relay::options::{FilterOptions, NegentropyOptions};
use crate::relay::{RelayOptions, RelaySendOptions, SubscribeOptions};
use crate::{HandleNotification, NostrDatabase, Relay};

#[derive(Object)]
pub struct RelayPool {
    inner: nostr_sdk::RelayPool,
}

#[uniffi::export]
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

    /// Start
    ///
    /// Internally call `connect` without wait for connection.
    #[inline]
    pub fn start(&self) {
        block_on(async move { self.inner.start().await })
    }

    /// Stop
    ///
    /// Call `connect` to re-start relays connections
    pub fn stop(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.stop().await?) })
    }

    /// Completely shutdown pool
    pub fn shutdown(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.clone().shutdown().await?) })
    }

    /// Get database
    pub fn database(&self) -> Arc<NostrDatabase> {
        Arc::new(self.inner.database().into())
    }

    /// Get relays
    pub fn relays(&self) -> HashMap<String, Arc<Relay>> {
        block_on(async move {
            self.inner
                .relays()
                .await
                .into_iter()
                .map(|(u, r)| (u.to_string(), Arc::new(r.into())))
                .collect()
        })
    }

    /// Get relay
    pub fn relay(&self, url: String) -> Result<Arc<Relay>> {
        block_on(async move { Ok(Arc::new(self.inner.relay(url).await?.into())) })
    }

    pub fn add_relay(&self, url: String, opts: &RelayOptions) -> Result<bool> {
        block_on(async move { Ok(self.inner.add_relay(url, opts.deref().clone()).await?) })
    }

    pub fn remove_relay(&self, url: String) -> Result<()> {
        block_on(async move { Ok(self.inner.remove_relay(url).await?) })
    }

    pub fn remove_all_relay(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.remove_all_relays().await?) })
    }

    /// Connect to all added relays and keep connection alive
    pub fn connect(&self, connection_timeout: Option<Duration>) {
        block_on(async move { self.inner.connect(connection_timeout).await })
    }

    /// Disconnect from all relays
    pub fn disconnect(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.disconnect().await?) })
    }

    /// Connect to relay
    pub fn connect_relay(&self, url: String, connection_timeout: Option<Duration>) -> Result<()> {
        block_on(async move { Ok(self.inner.connect_relay(url, connection_timeout).await?) })
    }

    /// Get subscriptions
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

    /// Send client message to all connected relays
    pub fn send_msg(&self, msg: Arc<ClientMessage>, opts: Arc<RelaySendOptions>) -> Result<()> {
        block_on(async move {
            Ok(self
                .inner
                .send_msg(msg.as_ref().deref().clone(), **opts)
                .await?)
        })
    }

    /// Send multiple client messages at once to all connected relays
    pub fn batch_msg(&self, msgs: Vec<Arc<ClientMessage>>, opts: &RelaySendOptions) -> Result<()> {
        let msgs = msgs
            .into_iter()
            .map(|msg| msg.as_ref().deref().clone())
            .collect();
        block_on(async move { Ok(self.inner.batch_msg(msgs, **opts).await?) })
    }

    /// Send client message to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub fn send_msg_to(
        &self,
        urls: Vec<String>,
        msg: Arc<ClientMessage>,
        opts: Arc<RelaySendOptions>,
    ) -> Result<()> {
        block_on(async move {
            Ok(self
                .inner
                .send_msg_to(urls, msg.as_ref().deref().clone(), **opts)
                .await?)
        })
    }

    /// Send multiple client messages at once to specific relays
    ///
    /// Note: **the relays must already be added!**
    pub fn batch_msg_to(
        &self,
        urls: Vec<String>,
        msgs: Vec<Arc<ClientMessage>>,
        opts: &RelaySendOptions,
    ) -> Result<()> {
        let msgs = msgs
            .into_iter()
            .map(|msg| msg.as_ref().deref().clone())
            .collect();
        block_on(async move { Ok(self.inner.batch_msg_to(urls, msgs, **opts).await?) })
    }

    /// Send event to **all connected relays** and wait for `OK` message
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

    /// Send multiple `Event` at once to **all connected relays** and wait for `OK` message
    pub fn batch_event(&self, events: Vec<Arc<Event>>, opts: &RelaySendOptions) -> Result<()> {
        let events = events
            .into_iter()
            .map(|e| e.as_ref().deref().clone())
            .collect();
        block_on(async move { Ok(self.inner.batch_event(events, **opts).await?) })
    }

    /// Send event to **specific relays** and wait for `OK` message
    pub fn send_event_to(
        &self,
        urls: Vec<String>,
        event: &Event,
        opts: &RelaySendOptions,
    ) -> Result<Arc<EventId>> {
        block_on(async move {
            Ok(Arc::new(
                self.inner
                    .send_event_to(urls, event.deref().clone(), **opts)
                    .await?
                    .into(),
            ))
        })
    }

    /// Send multiple events at once to **specific relays** and wait for `OK` message
    pub fn batch_event_to(
        &self,
        urls: Vec<String>,
        events: Vec<Arc<Event>>,
        opts: &RelaySendOptions,
    ) -> Result<()> {
        let events = events
            .into_iter()
            .map(|e| e.as_ref().deref().clone())
            .collect();
        block_on(async move { Ok(self.inner.batch_event_to(urls, events, **opts).await?) })
    }

    /// Subscribe to filters to all connected relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    ///
    /// Note: auto-closing subscriptions aren't saved in subscriptions map!
    pub fn subscribe(&self, filters: Vec<Arc<Filter>>, opts: &SubscribeOptions) -> String {
        block_on(async move {
            self.inner
                .subscribe(
                    filters
                        .into_iter()
                        .map(|f| f.as_ref().deref().clone())
                        .collect(),
                    **opts,
                )
                .await
                .to_string()
        })
    }

    /// Subscribe with custom subscription ID to all connected relays
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
    ) {
        block_on(async move {
            self.inner
                .subscribe_with_id(
                    SubscriptionId::new(id),
                    filters
                        .into_iter()
                        .map(|f| f.as_ref().deref().clone())
                        .collect(),
                    **opts,
                )
                .await
        })
    }

    /// Subscribe to filters to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    pub fn subscribe_to(
        &self,
        urls: Vec<String>,
        filters: Vec<Arc<Filter>>,
        opts: &SubscribeOptions,
    ) -> Result<String> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        block_on(async move {
            Ok(self
                .inner
                .subscribe_to(urls, filters, **opts)
                .await?
                .to_string())
        })
    }

    /// Subscribe to filters with custom subscription ID to specific relays
    ///
    /// ### Auto-closing subscription
    ///
    /// It's possible to automatically close a subscription by configuring the `SubscribeOptions`.
    pub fn subscribe_with_id_to(
        &self,
        urls: Vec<String>,
        id: String,
        filters: Vec<Arc<Filter>>,
        opts: &SubscribeOptions,
    ) -> Result<()> {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        block_on(async move {
            Ok(self
                .inner
                .subscribe_with_id_to(urls, SubscriptionId::new(id), filters, **opts)
                .await?)
        })
    }

    /// Unsubscribe
    pub fn unsubscribe(&self, id: String, opts: Arc<RelaySendOptions>) {
        block_on(async move {
            self.inner
                .unsubscribe(SubscriptionId::new(id), **opts)
                .await
        })
    }

    /// Unsubscribe from all subscriptions
    pub fn unsubscribe_all(&self, opts: Arc<RelaySendOptions>) {
        block_on(async move { self.inner.unsubscribe_all(**opts).await })
    }

    /// Get events of filters
    ///
    /// Get events both from **local database** and **relays**
    pub fn get_events_of(
        &self,
        filters: Vec<Arc<Filter>>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Arc<Event>>> {
        block_on(async move {
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
        })
    }

    /// Get events of filters from **specific relays**
    ///
    /// Get events both from **local database** and **relays**
    pub fn get_events_from(
        &self,
        urls: Vec<String>,
        filters: Vec<Arc<Filter>>,
        timeout: Duration,
        opts: FilterOptions,
    ) -> Result<Vec<Arc<Event>>> {
        block_on(async move {
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

    /// Handle relay pool notifications
    pub fn handle_notifications(
        self: Arc<Self>,
        handler: Box<dyn HandleNotification>,
    ) -> Result<()> {
        thread::spawn(async move {
            let handler = Arc::new(handler);
            self.inner
                .handle_notifications(|notification| async {
                    match notification {
                        nostr_sdk::RelayPoolNotification::Message { relay_url, message } => {
                            let h = handler.clone();
                            let _ = spawn_blocking(move || {
                                h.handle_msg(relay_url.to_string(), Arc::new(message.into()))
                            })
                            .await;
                        }
                        nostr_sdk::RelayPoolNotification::Event {
                            relay_url,
                            subscription_id,
                            event,
                        } => {
                            let h = handler.clone();
                            let _ = spawn_blocking(move || {
                                h.handle(
                                    relay_url.to_string(),
                                    subscription_id.to_string(),
                                    Arc::new((*event).into()),
                                )
                            })
                            .await;
                        }
                        _ => (),
                    }
                    Ok(false)
                })
                .await
        })?;
        Ok(())
    }
}
