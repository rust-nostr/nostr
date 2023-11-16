// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::client::options::DEFAULT_SEND_TIMEOUT;

pub const DEFAULT_RETRY_SEC: u64 = 10;
pub const MIN_RETRY_SEC: u64 = 5;
pub const MAX_ADJ_RETRY_SEC: u64 = 60;

/// [`Relay`](super::Relay) options
#[derive(Debug, Clone)]
pub struct RelayOptions {
    /// Allow/disallow read actions (default: true)
    read: Arc<AtomicBool>,
    /// Allow/disallow write actions (default: true)
    write: Arc<AtomicBool>,
    /// Enable/disable auto reconnection (default: true)
    reconnect: Arc<AtomicBool>,
    /// Retry connection time (default: 10 sec)
    ///
    /// Are allowed values `>=` 5 secs
    retry_sec: Arc<AtomicU64>,
    /// Automatically adjust retry seconds based on success/attempts (default: true)
    adjust_retry_sec: Arc<AtomicBool>,
}

impl Default for RelayOptions {
    fn default() -> Self {
        Self {
            read: Arc::new(AtomicBool::new(true)),
            write: Arc::new(AtomicBool::new(true)),
            reconnect: Arc::new(AtomicBool::new(true)),
            retry_sec: Arc::new(AtomicU64::new(DEFAULT_RETRY_SEC)),
            adjust_retry_sec: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl RelayOptions {
    /// New [`RelayOptions`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set read option
    pub fn read(self, read: bool) -> Self {
        Self {
            read: Arc::new(AtomicBool::new(read)),
            ..self
        }
    }

    pub(crate) fn get_read(&self) -> bool {
        self.read.load(Ordering::SeqCst)
    }

    /// Update read option
    pub fn update_read(&self, read: bool) {
        let _ = self
            .read
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(read));
    }

    /// Set write option
    pub fn write(self, write: bool) -> Self {
        Self {
            write: Arc::new(AtomicBool::new(write)),
            ..self
        }
    }

    pub(crate) fn get_write(&self) -> bool {
        self.write.load(Ordering::SeqCst)
    }

    /// Update write option
    pub fn update_write(&self, write: bool) {
        let _ = self
            .write
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(write));
    }

    /// Set reconnect option
    pub fn reconnect(self, reconnect: bool) -> Self {
        Self {
            reconnect: Arc::new(AtomicBool::new(reconnect)),
            ..self
        }
    }

    pub(crate) fn get_reconnect(&self) -> bool {
        self.reconnect.load(Ordering::SeqCst)
    }

    /// Set `reconnect` option
    pub fn update_reconnect(&self, reconnect: bool) {
        let _ = self
            .reconnect
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(reconnect));
    }

    /// Set retry seconds option
    pub fn retry_sec(self, retry_sec: u64) -> Self {
        let retry_sec = if retry_sec >= MIN_RETRY_SEC {
            retry_sec
        } else {
            DEFAULT_RETRY_SEC
        };
        Self {
            retry_sec: Arc::new(AtomicU64::new(retry_sec)),
            ..self
        }
    }

    pub(crate) fn get_retry_sec(&self) -> u64 {
        self.retry_sec.load(Ordering::SeqCst)
    }

    /// Set retry_sec option
    pub fn update_retry_sec(&self, retry_sec: u64) {
        if retry_sec >= MIN_RETRY_SEC {
            let _ = self
                .retry_sec
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(retry_sec));
        } else {
            tracing::warn!("Relay options: retry_sec it's less then the minimum value allowed (min: {MIN_RETRY_SEC} secs)");
        }
    }

    /// Automatically adjust retry seconds based on success/attempts (default: true)
    pub fn adjust_retry_sec(self, adjust_retry_sec: bool) -> Self {
        Self {
            adjust_retry_sec: Arc::new(AtomicBool::new(adjust_retry_sec)),
            ..self
        }
    }

    pub(crate) fn get_adjust_retry_sec(&self) -> bool {
        self.adjust_retry_sec.load(Ordering::SeqCst)
    }

    /// Set adjust_retry_sec option
    pub fn update_adjust_retry_sec(&self, adjust_retry_sec: bool) {
        let _ = self
            .adjust_retry_sec
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| {
                Some(adjust_retry_sec)
            });
    }
}

/// [`Relay`](super::Relay) send options
#[derive(Debug, Clone, Copy)]
pub struct RelaySendOptions {
    /// Skip wait for disconnected relay (default: true)
    pub skip_disconnected: bool,
    /// Timeout for sending event (default: 10 secs)
    pub timeout: Duration,
}

impl Default for RelaySendOptions {
    fn default() -> Self {
        Self {
            skip_disconnected: true,
            timeout: DEFAULT_SEND_TIMEOUT,
        }
    }
}

impl RelaySendOptions {
    /// New default [`RelaySendOptions`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Skip wait for disconnected relay (default: true)
    pub fn skip_disconnected(self, value: bool) -> Self {
        Self {
            skip_disconnected: value,
            ..self
        }
    }

    /// Timeout for sending event (default: 10 secs)
    ///
    /// If `None`, the default timeout will be used
    pub fn timeout(self, value: Option<Duration>) -> Self {
        Self {
            timeout: value.unwrap_or(DEFAULT_SEND_TIMEOUT),
            ..self
        }
    }
}

/// Filter options
#[derive(Debug, Clone, Copy, Default)]
pub enum FilterOptions {
    /// Exit on EOSE
    #[default]
    ExitOnEOSE,
    /// After EOSE is received, keep listening for N more events that match the filter, then return
    WaitForEventsAfterEOSE(u16),
    /// After EOSE is received, keep listening for matching events for [`Duration`] more time, then return
    WaitDurationAfterEOSE(Duration),
}

/// Relay Pool Options
#[derive(Debug, Clone, Copy)]
pub struct RelayPoolOptions {
    /// Notification channel size (default: 1024)
    pub notification_channel_size: usize,
    /// Task channel size (default: 1024)
    pub task_channel_size: usize,
    /// Shutdown on [RelayPool](super::pool::RelayPool) drop
    pub shutdown_on_drop: bool,
}

impl Default for RelayPoolOptions {
    fn default() -> Self {
        Self {
            notification_channel_size: 1024,
            task_channel_size: 1024,
            shutdown_on_drop: false,
        }
    }
}

impl RelayPoolOptions {
    /// New default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Shutdown on [`RelayPool`](super::pool::RelayPool) drop
    pub fn shutdown_on_drop(self, value: bool) -> Self {
        Self {
            shutdown_on_drop: value,
            ..self
        }
    }
}

/// Negentropy reconciliation options
#[derive(Debug, Clone, Copy)]
pub struct NegentropyOptions {
    /// Total timeout for reconciliation (default: 60 secs)
    pub timeout: Duration,
    /// Timeout for **single** reconciliation (default: 30 secs)
    ///
    /// Should be less than total timeout
    pub single_reconciliation_timeout: Duration,
    /// Syncronous (default: true)
    ///
    /// If `true`, request events and wait that relay send them.
    /// If `false`, request events but continue the reconciliation
    pub syncrounous: bool,
    /// Bidirectional Sync (default: false)
    ///
    /// If `true`, perform the set reconciliation on each side.
    pub bidirectional: bool,
}

impl Default for NegentropyOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            single_reconciliation_timeout: Duration::from_secs(30),
            syncrounous: true,
            bidirectional: false,
        }
    }
}

impl NegentropyOptions {
    /// New default [`NegentropyOptions`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Timeout for sending event (default: 60 secs)
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Timeout for **single** reconciliation (default: 30 secs)
    ///
    /// Should be less than total timeout
    pub fn single_reconciliation_timeout(
        mut self,
        single_reconciliation_timeout: Duration,
    ) -> Self {
        self.single_reconciliation_timeout = single_reconciliation_timeout;
        self
    }

    /// Syncronous (default: true)
    ///
    /// If `true`, request events and wait that relay send them.
    /// If `false`, request events but continue the reconciliation
    pub fn syncrounous(mut self, syncrounous: bool) -> Self {
        self.syncrounous = syncrounous;
        self
    }

    /// Bidirectional Sync (default: false)
    ///
    /// If `true`, perform the set reconciliation on each side.
    pub fn bidirectional(mut self, bidirectional: bool) -> Self {
        self.bidirectional = bidirectional;
        self
    }
}
