// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Monitor

use nostr::RelayUrl;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::relay::RelayStatus;

/// Relay monitor notification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MonitorNotification {
    /// Relay status changed
    StatusChanged {
        /// Relay URL
        relay_url: RelayUrl,
        /// Status
        status: RelayStatus,
    },
}

/// Relay monitor
#[derive(Debug, Clone)]
pub struct Monitor {
    channel: Sender<MonitorNotification>,
}

impl Monitor {
    /// Create a new monitor with the given channel size
    ///
    /// For more details, check [`broadcast::channel`].
    pub fn new(channel_size: usize) -> Self {
        let (tx, ..) = broadcast::channel(channel_size);

        Self { channel: tx }
    }

    /// Subscribe to monitor notifications
    ///
    /// <div class="warning">When you call this method, you subscribe to the notifications channel from that precise moment. Anything received by relay/s before that moment is not included in the channel!</div>
    #[inline]
    pub fn subscribe(&self) -> Receiver<MonitorNotification> {
        self.channel.subscribe()
    }

    #[inline]
    fn notify(&self, notification: MonitorNotification) {
        let _ = self.channel.send(notification);
    }

    #[inline]
    pub(crate) fn notify_status_change(&self, relay_url: RelayUrl, status: RelayStatus) {
        self.notify(MonitorNotification::StatusChanged { relay_url, status });
    }
}
