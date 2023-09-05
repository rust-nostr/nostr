// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::client::options::DEFAULT_SEND_TIMEOUT;

/// [`Relay`](super::Relay) options
#[derive(Debug, Clone)]
pub struct RelayOptions {
    /// Allow/disallow read actions
    read: Arc<AtomicBool>,
    /// Allow/disallow write actions
    write: Arc<AtomicBool>,
}

impl Default for RelayOptions {
    fn default() -> Self {
        Self::new(true, true)
    }
}

impl RelayOptions {
    /// New [`RelayOptions`]
    pub fn new(read: bool, write: bool) -> Self {
        Self {
            read: Arc::new(AtomicBool::new(read)),
            write: Arc::new(AtomicBool::new(write)),
        }
    }

    /// Get read option
    pub fn read(&self) -> bool {
        self.read.load(Ordering::SeqCst)
    }

    /// Set read option
    pub fn set_read(&self, read: bool) {
        let _ = self
            .read
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(read));
    }

    /// Get write option
    pub fn write(&self) -> bool {
        self.write.load(Ordering::SeqCst)
    }

    /// Set write option
    pub fn set_write(&self, write: bool) {
        let _ = self
            .write
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(write));
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
    /// Max seen events by Task thread (default: 1_000_000)
    ///
    /// A lower number can cause receiving in notification channel
    /// the same event multiple times
    pub task_max_seen_events: usize,
    /// Shutdown on [RelayPool](super::pool::RelayPool) drop
    pub shutdown_on_drop: bool,
}

impl Default for RelayPoolOptions {
    fn default() -> Self {
        Self {
            notification_channel_size: 1024,
            task_channel_size: 1024,
            task_max_seen_events: 1_000_000,
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
