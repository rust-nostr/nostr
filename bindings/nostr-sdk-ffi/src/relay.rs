// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, ops::Deref};

use nostr_ffi::{ClientMessage, Event, Filter, RelayInformationDocument};
use nostr_sdk::relay::InternalSubscriptionId;
use nostr_sdk::{block_on, relay, FilterOptions, RelayStatus};

use crate::error::Result;

pub struct RelayConnectionStats {
    inner: relay::RelayConnectionStats,
}

impl From<relay::RelayConnectionStats> for RelayConnectionStats {
    fn from(inner: relay::RelayConnectionStats) -> Self {
        Self { inner }
    }
}

impl RelayConnectionStats {
    pub fn attempts(&self) -> u64 {
        self.inner.attempts() as u64
    }

    pub fn success(&self) -> u64 {
        self.inner.success() as u64
    }

    pub fn connected_at(&self) -> u64 {
        self.inner.connected_at().as_u64()
    }
}

pub struct ActiveSubscription {
    inner: relay::ActiveSubscription,
}

impl From<relay::ActiveSubscription> for ActiveSubscription {
    fn from(inner: relay::ActiveSubscription) -> Self {
        Self { inner }
    }
}

impl ActiveSubscription {
    pub fn id(&self) -> String {
        self.inner.id().to_string()
    }

    pub fn filters(&self) -> Vec<Arc<Filter>> {
        self.inner
            .filters()
            .into_iter()
            .map(|f| Arc::new(f.into()))
            .collect()
    }
}

pub struct Relay {
    inner: relay::Relay,
}

impl From<relay::Relay> for Relay {
    fn from(inner: relay::Relay) -> Self {
        Self { inner }
    }
}

impl Relay {
    pub fn url(&self) -> String {
        self.inner.url().to_string()
    }

    pub fn proxy(&self) -> Option<String> {
        self.inner.proxy().map(|p| p.to_string())
    }

    pub fn status(&self) -> RelayStatus {
        self.inner.status_blocking()
    }

    pub fn is_connected(&self) -> bool {
        block_on(async move { self.inner.is_connected().await })
    }

    pub fn document(&self) -> Arc<RelayInformationDocument> {
        Arc::new(self.inner.document_blocking().into())
    }

    pub fn subscriptions(&self) -> HashMap<String, Arc<ActiveSubscription>> {
        block_on(async move {
            self.inner
                .subscriptions()
                .await
                .into_iter()
                .map(|(id, sub)| (id.to_string(), Arc::new(sub.into())))
                .collect()
        })
    }

    pub fn update_subscription_filters(&self, internal_id: String, filters: Vec<Arc<Filter>>) {
        block_on(
            self.inner.update_subscription_filters(
                InternalSubscriptionId::Custom(internal_id),
                filters
                    .into_iter()
                    .map(|f| f.as_ref().deref().clone())
                    .collect(),
            ),
        )
    }

    // TODO: add opts

    pub fn stats(&self) -> Arc<RelayConnectionStats> {
        Arc::new(self.inner.stats().into())
    }

    pub fn queue(&self) -> u64 {
        self.inner.queue() as u64
    }

    pub fn connect(&self, wait_for_connection: bool) {
        block_on(self.inner.connect(wait_for_connection))
    }

    pub fn stop(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.stop().await?) })
    }

    pub fn terminate(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.terminate().await?) })
    }

    pub fn send_msg(&self, msg: ClientMessage, wait: Option<Duration>) -> Result<()> {
        block_on(async move { Ok(self.inner.send_msg(msg.try_into()?, wait).await?) })
    }

    pub fn subscribe(&self, filters: Vec<Arc<Filter>>, wait: Option<Duration>) -> Result<()> {
        block_on(async move {
            Ok(self
                .inner
                .subscribe(
                    filters
                        .into_iter()
                        .map(|f| f.as_ref().deref().clone())
                        .collect(),
                    wait,
                )
                .await?)
        })
    }

    pub fn unsubscribe(&self, wait: Option<Duration>) -> Result<()> {
        block_on(async move { Ok(self.inner.unsubscribe(wait).await?) })
    }

    pub fn get_events_of(
        &self,
        filters: Vec<Arc<Filter>>,
        timeout: Option<Duration>,
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

    pub fn req_events_of(&self, filters: Vec<Arc<Filter>>, timeout: Option<Duration>) {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        self.inner
            .req_events_of(filters, timeout, FilterOptions::ExitOnEOSE);
    }
}
