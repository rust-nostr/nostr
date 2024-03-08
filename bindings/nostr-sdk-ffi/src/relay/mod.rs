// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use nostr_ffi::{ClientMessage, Event, Filter, RelayInformationDocument, Timestamp};
use nostr_sdk::{block_on, pool, FilterOptions, SubscriptionId};
use uniffi::{Enum, Object};

pub mod options;

use self::options::RelaySendOptions;
use crate::error::Result;

#[derive(Object)]
pub struct RelayConnectionStats {
    inner: pool::RelayConnectionStats,
}

impl From<pool::RelayConnectionStats> for RelayConnectionStats {
    fn from(inner: pool::RelayConnectionStats) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl RelayConnectionStats {
    pub fn attempts(&self) -> u64 {
        self.inner.attempts() as u64
    }

    pub fn success(&self) -> u64 {
        self.inner.success() as u64
    }

    pub fn uptime(&self) -> f64 {
        self.inner.uptime()
    }

    pub fn connected_at(&self) -> Timestamp {
        let secs = self.inner.connected_at().as_u64();
        Timestamp::from_secs(secs)
    }

    pub fn bytes_sent(&self) -> u64 {
        self.inner.bytes_sent() as u64
    }

    pub fn bytes_received(&self) -> u64 {
        self.inner.bytes_received() as u64
    }

    pub fn latency(&self) -> Option<Duration> {
        block_on(async move { self.inner.latency().await })
    }
}

#[derive(Enum)]
pub enum RelayStatus {
    /// Relay initialized
    Initialized,
    /// Pending
    Pending,
    /// Connecting
    Connecting,
    /// Relay connected
    Connected,
    /// Relay disconnected, will retry to connect again
    Disconnected,
    /// Stop
    Stopped,
    /// Relay completely disconnected
    Terminated,
}

impl From<nostr_sdk::RelayStatus> for RelayStatus {
    fn from(value: nostr_sdk::RelayStatus) -> Self {
        match value {
            nostr_sdk::RelayStatus::Initialized => Self::Initialized,
            nostr_sdk::RelayStatus::Pending => Self::Pending,
            nostr_sdk::RelayStatus::Connecting => Self::Connecting,
            nostr_sdk::RelayStatus::Connected => Self::Connected,
            nostr_sdk::RelayStatus::Disconnected => Self::Disconnected,
            nostr_sdk::RelayStatus::Stopped => Self::Stopped,
            nostr_sdk::RelayStatus::Terminated => Self::Terminated,
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

#[uniffi::export]
impl Relay {
    pub fn url(&self) -> String {
        self.inner.url().to_string()
    }

    pub fn proxy(&self) -> Option<String> {
        self.inner.proxy().map(|p| p.to_string())
    }

    pub fn status(&self) -> RelayStatus {
        block_on(async move { self.inner.status().await.into() })
    }

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

    // TODO: add opts

    pub fn stats(&self) -> Arc<RelayConnectionStats> {
        Arc::new(self.inner.stats().into())
    }

    pub fn queue(&self) -> u64 {
        self.inner.queue() as u64
    }

    pub fn connect(&self, connection_timeout: Option<Duration>) {
        block_on(self.inner.connect(connection_timeout))
    }

    pub fn stop(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.stop().await?) })
    }

    pub fn terminate(&self) -> Result<()> {
        block_on(async move { Ok(self.inner.terminate().await?) })
    }

    pub fn send_msg(&self, msg: Arc<ClientMessage>, opts: Arc<RelaySendOptions>) -> Result<()> {
        block_on(async move {
            Ok(self
                .inner
                .send_msg(msg.as_ref().deref().clone(), **opts)
                .await?)
        })
    }

    pub fn subscribe(
        &self,
        filters: Vec<Arc<Filter>>,
        opts: Arc<RelaySendOptions>,
    ) -> Result<String> {
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

    pub fn subscribe_with_id(
        &self,
        id: String,
        filters: Vec<Arc<Filter>>,
        opts: Arc<RelaySendOptions>,
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

    pub fn unsubscribe(&self, id: String, opts: Arc<RelaySendOptions>) -> Result<()> {
        block_on(async move {
            Ok(self
                .inner
                .unsubscribe(SubscriptionId::new(id), **opts)
                .await?)
        })
    }

    pub fn unsubscribe_all(&self, opts: Arc<RelaySendOptions>) -> Result<()> {
        block_on(async move { Ok(self.inner.unsubscribe_all(**opts).await?) })
    }

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

    pub fn req_events_of(&self, filters: Vec<Arc<Filter>>, timeout: Duration) {
        let filters = filters
            .into_iter()
            .map(|f| f.as_ref().deref().clone())
            .collect();
        self.inner
            .req_events_of(filters, timeout, FilterOptions::ExitOnEOSE);
    }
}
