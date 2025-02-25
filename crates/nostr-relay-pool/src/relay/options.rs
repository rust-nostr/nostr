// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay options

use std::time::Duration;

use async_wsocket::ConnectionMode;
use tokio::sync::watch::{self, Receiver, Sender};

use super::constants::DEFAULT_RETRY_INTERVAL;
use super::flags::RelayServiceFlags;
use crate::RelayLimits;

/// Relay options
#[derive(Debug, Clone)]
pub struct RelayOptions {
    pub(super) connection_mode: ConnectionMode,
    pub(super) flags: RelayServiceFlags,
    pub(super) reconnect: bool,
    pub(super) retry_interval: Duration,
    pub(super) adjust_retry_interval: bool,
    pub(super) limits: RelayLimits,
    pub(super) max_avg_latency: Option<Duration>,
}

impl Default for RelayOptions {
    fn default() -> Self {
        Self {
            connection_mode: ConnectionMode::default(),
            flags: RelayServiceFlags::default(),
            reconnect: true,
            retry_interval: DEFAULT_RETRY_INTERVAL,
            adjust_retry_interval: true,
            limits: RelayLimits::default(),
            max_avg_latency: None,
        }
    }
}

impl RelayOptions {
    /// New default options
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set connection mode
    #[inline]
    pub fn connection_mode(mut self, mode: ConnectionMode) -> Self {
        self.connection_mode = mode;
        self
    }

    /// Set Relay Service Flags
    pub fn flags(mut self, flags: RelayServiceFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set read flag
    pub fn read(mut self, read: bool) -> Self {
        if read {
            self.flags.add(RelayServiceFlags::READ);
        } else {
            self.flags.remove(RelayServiceFlags::READ);
        }
        self
    }

    /// Set write flag
    pub fn write(mut self, write: bool) -> Self {
        if write {
            self.flags.add(RelayServiceFlags::WRITE);
        } else {
            self.flags.remove(RelayServiceFlags::WRITE);
        }
        self
    }

    /// Set ping flag
    pub fn ping(mut self, ping: bool) -> Self {
        if ping {
            self.flags.add(RelayServiceFlags::PING);
        } else {
            self.flags.remove(RelayServiceFlags::PING);
        }
        self
    }

    /// Enable/disable auto reconnection (default: true)
    pub fn reconnect(mut self, reconnect: bool) -> Self {
        self.reconnect = reconnect;
        self
    }

    /// Retry connection time (default: 10 sec)
    pub fn retry_interval(mut self, interval: Duration) -> Self {
        self.retry_interval = interval;
        self
    }

    /// Automatically adjust retry interval based on success/attempts (default: true)
    pub fn adjust_retry_interval(mut self, adjust_retry_interval: bool) -> Self {
        self.adjust_retry_interval = adjust_retry_interval;
        self
    }

    /// Set custom limits
    pub fn limits(mut self, limits: RelayLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Set max latency (default: None)
    ///
    /// Relay with an avg. latency greater that this value will be skipped.
    #[inline]
    pub fn max_avg_latency(mut self, max: Option<Duration>) -> Self {
        self.max_avg_latency = max;
        self
    }
}

/// Auto-closing subscribe options
#[derive(Debug, Clone, Copy, Default)]
pub struct SubscribeAutoCloseOptions {
    pub(super) exit_policy: ReqExitPolicy,
    pub(super) timeout: Option<Duration>,
    pub(super) idle_timeout: Option<Duration>,
}

impl SubscribeAutoCloseOptions {
    /// Close subscription when [`ReqExitPolicy`] is satisfied
    pub fn exit_policy(mut self, policy: ReqExitPolicy) -> Self {
        self.exit_policy = policy;
        self
    }

    /// Automatically close subscription after [`Duration`].
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Automatically close subscription if no notifications/events are received within the [`Duration`].
    pub fn idle_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.idle_timeout = timeout;
        self
    }
}

/// Subscribe options
#[derive(Debug, Clone, Copy, Default)]
pub struct SubscribeOptions {
    pub(super) auto_close: Option<SubscribeAutoCloseOptions>,
}

impl SubscribeOptions {
    /// Set auto-close conditions
    pub fn close_on(mut self, opts: Option<SubscribeAutoCloseOptions>) -> Self {
        self.auto_close = opts;
        self
    }

    pub(crate) fn is_auto_closing(&self) -> bool {
        self.auto_close.is_some()
    }
}

/// Request (REQ) exit policy
#[derive(Debug, Clone, Copy, Default)]
pub enum ReqExitPolicy {
    /// Exit on EOSE.
    #[default]
    ExitOnEOSE,
    /// After EOSE is received, keep listening for N more events that match the filter.
    WaitForEventsAfterEOSE(u16),
    /// After EOSE is received, keep listening for matching events for [`Duration`] more time.
    WaitDurationAfterEOSE(Duration),
}

/// Negentropy Sync direction
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SyncDirection {
    /// Send events to relay
    Up,
    /// Get events from relay
    #[default]
    Down,
    /// Both send and get events from relay (bidirectional sync)
    Both,
}

/// Sync (negentropy reconciliation) progress
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SyncProgress {
    /// Total events to process
    pub total: u64,
    /// Processed events
    pub current: u64,
}

impl SyncProgress {
    /// Construct new sync progress channel
    #[inline]
    pub fn channel() -> (Sender<Self>, Receiver<Self>) {
        watch::channel(SyncProgress::default())
    }

    /// Calculate progress %
    #[inline]
    pub fn percentage(&self) -> f64 {
        if self.total > 0 {
            self.current as f64 / self.total as f64
        } else {
            0.0
        }
    }
}

/// Sync (negentropy reconciliation) options
#[derive(Debug, Clone)]
pub struct SyncOptions {
    pub(super) initial_timeout: Duration,
    pub(super) direction: SyncDirection,
    pub(super) dry_run: bool,
    pub(super) progress: Option<Sender<SyncProgress>>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            initial_timeout: Duration::from_secs(10),
            direction: SyncDirection::default(),
            dry_run: false,
            progress: None,
        }
    }
}

impl SyncOptions {
    /// New default [`SyncOptions`]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Timeout to check if negentropy it's supported (default: 10 secs)
    #[inline]
    pub fn initial_timeout(mut self, initial_timeout: Duration) -> Self {
        self.initial_timeout = initial_timeout;
        self
    }

    /// Negentropy Sync direction (default: down)
    ///
    /// If `true`, perform the set reconciliation on each side.
    #[inline]
    pub fn direction(mut self, direction: SyncDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Dry run
    ///
    /// Just check what event are missing: execute reconciliation but WITHOUT
    /// getting/sending full events.
    #[inline]
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Sync progress
    ///
    /// Use [`SyncProgress::channel`] to create a watch channel and pass the sender here.
    #[inline]
    pub fn progress(mut self, sender: Sender<SyncProgress>) -> Self {
        self.progress = Some(sender);
        self
    }

    #[inline]
    pub(super) fn do_up(&self) -> bool {
        !self.dry_run && matches!(self.direction, SyncDirection::Up | SyncDirection::Both)
    }

    #[inline]
    pub(super) fn do_down(&self) -> bool {
        !self.dry_run && matches!(self.direction, SyncDirection::Down | SyncDirection::Both)
    }
}
