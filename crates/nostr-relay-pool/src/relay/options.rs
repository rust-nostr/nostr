// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay options

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_wsocket::ConnectionMode;
use tokio::sync::watch::{self, Receiver, Sender};

use super::constants::{DEFAULT_RETRY_SEC, DEFAULT_SEND_TIMEOUT, MIN_RETRY_SEC};
use super::filtering::RelayFilteringMode;
use super::flags::{AtomicRelayServiceFlags, RelayServiceFlags};
use crate::RelayLimits;

/// [`Relay`](super::Relay) options
#[derive(Debug, Clone)]
pub struct RelayOptions {
    pub(super) connection_mode: ConnectionMode,
    pub(super) flags: AtomicRelayServiceFlags,
    pow: Arc<AtomicU8>,
    reconnect: Arc<AtomicBool>,
    retry_sec: Arc<AtomicU64>,
    adjust_retry_sec: Arc<AtomicBool>,
    pub(super) limits: RelayLimits,
    pub(super) max_avg_latency: Option<Duration>,
    pub(super) filtering_mode: RelayFilteringMode,
}

impl Default for RelayOptions {
    fn default() -> Self {
        Self {
            connection_mode: ConnectionMode::default(),
            flags: AtomicRelayServiceFlags::default(),
            pow: Arc::new(AtomicU8::new(0)),
            reconnect: Arc::new(AtomicBool::new(true)),
            retry_sec: Arc::new(AtomicU64::new(DEFAULT_RETRY_SEC)),
            adjust_retry_sec: Arc::new(AtomicBool::new(true)),
            limits: RelayLimits::default(),
            max_avg_latency: None,
            filtering_mode: RelayFilteringMode::default(),
        }
    }
}

impl RelayOptions {
    /// New [`RelayOptions`]
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
        self.flags = AtomicRelayServiceFlags::new(flags);
        self
    }

    /// Set read flag
    pub fn read(self, read: bool) -> Self {
        if read {
            self.flags.add(RelayServiceFlags::READ);
        } else {
            self.flags.remove(RelayServiceFlags::READ);
        }
        self
    }

    /// Set write flag
    pub fn write(self, write: bool) -> Self {
        if write {
            self.flags.add(RelayServiceFlags::WRITE);
        } else {
            self.flags.remove(RelayServiceFlags::WRITE);
        }
        self
    }

    /// Set ping flag
    pub fn ping(self, ping: bool) -> Self {
        if ping {
            self.flags.add(RelayServiceFlags::PING);
        } else {
            self.flags.remove(RelayServiceFlags::PING);
        }
        self
    }

    /// Minimum POW for received events (default: 0)
    pub fn pow(mut self, difficulty: u8) -> Self {
        self.pow = Arc::new(AtomicU8::new(difficulty));
        self
    }

    pub(crate) fn get_pow_difficulty(&self) -> u8 {
        self.pow.load(Ordering::SeqCst)
    }

    /// Set `pow` option
    pub fn update_pow_difficulty(&self, difficulty: u8) {
        self.pow.store(difficulty, Ordering::SeqCst);
    }

    /// Enable/disable auto reconnection (default: true)
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
        self.reconnect.store(reconnect, Ordering::SeqCst);
    }

    /// Retry connection time (default: 10 sec)
    ///
    /// Are allowed values `>=` 5 secs
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
            self.retry_sec.store(retry_sec, Ordering::SeqCst);
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
        self.adjust_retry_sec
            .store(adjust_retry_sec, Ordering::SeqCst);
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

    /// Relay filtering mode (default: blacklist)
    #[inline]
    pub fn filtering_mode(mut self, mode: RelayFilteringMode) -> Self {
        self.filtering_mode = mode;
        self
    }
}

/// [`Relay`](super::Relay) send options
#[derive(Debug, Clone, Copy)]
pub struct RelaySendOptions {
    pub(super) timeout: Duration,
}

impl Default for RelaySendOptions {
    fn default() -> Self {
        Self {
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
    #[deprecated(
        since = "0.36.0",
        note = "Disconnected relays will be skipped by default"
    )]
    pub fn skip_disconnected(self, _value: bool) -> Self {
        self
    }

    /// Skip wait for confirmation that message is sent (default: false)
    #[deprecated(since = "0.36.0", note = "By default confirmation will not be skipped")]
    pub fn skip_send_confirmation(self, _value: bool) -> Self {
        self
    }

    /// Timeout for sending event (default: 20 secs)
    ///
    /// If `None`, the default timeout will be used
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout.unwrap_or(DEFAULT_SEND_TIMEOUT);
        self
    }
}

/// Auto-closing subscribe options
#[derive(Debug, Clone, Copy, Default)]
pub struct SubscribeAutoCloseOptions {
    pub(super) filter: FilterOptions,
    pub(super) timeout: Option<Duration>,
}

impl SubscribeAutoCloseOptions {
    /// Close subscription when [FilterOptions] is satisfied
    pub fn filter(mut self, filter: FilterOptions) -> Self {
        self.filter = filter;
        self
    }

    /// Automatically close subscription after [Duration]
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
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

    /// Set [RelaySendOptions]
    #[deprecated(since = "0.36.0")]
    pub fn send_opts(self, _opts: RelaySendOptions) -> Self {
        self
    }

    pub(crate) fn is_auto_closing(&self) -> bool {
        self.auto_close.is_some()
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
