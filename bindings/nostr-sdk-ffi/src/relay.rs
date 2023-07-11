// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::{Event, Filter};
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

    // TODO: add NIP11 document

    pub fn subscription(&self) -> Arc<ActiveSubscription> {
        block_on(async move { Arc::new(self.inner.subscription().await.into()) })
    }

    pub fn update_subscription_filters(&self, filters: Vec<Arc<Filter>>) {
        block_on(
            self.inner.update_subscription_filters(
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

    pub fn connect(&self, wait_for_connection: bool) {
        block_on(self.inner.connect(wait_for_connection))
    }

    pub fn stop(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.stop().await?) })
    }

    pub fn terminate(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.terminate().await?) })
    }

    // TODO: add send_msg

    pub fn subscribe(&self, filters: Vec<Arc<Filter>>, wait: Option<Duration>) -> Result<String> {
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
                .await?
                .to_string())
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
