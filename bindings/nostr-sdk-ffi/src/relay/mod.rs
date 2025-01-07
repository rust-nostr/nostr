// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::sync::Arc;

use nostr_sdk::pool;
use uniffi::{Object, Record};

pub mod filtering;
pub mod limits;
pub mod options;
pub mod stats;
pub mod status;

pub use self::filtering::{RelayFiltering, RelayFilteringMode};
pub use self::limits::RelayLimits;
pub use self::options::{ConnectionMode, RelayOptions, ReqExitPolicy, SubscribeOptions};
pub use self::stats::RelayConnectionStats;
pub use self::status::RelayStatus;
use crate::protocol::{EventId, RelayInformationDocument};

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

    pub fn opts(&self) -> RelayOptions {
        self.inner.opts().clone().into()
    }

    pub fn stats(&self) -> RelayConnectionStats {
        self.inner.stats().clone().into()
    }
}
