// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay options

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_wsocket::ConnectionMode;

use super::filtering::RelayFilteringMode;
use super::flags::{AtomicRelayServiceFlags, RelayServiceFlags};
use crate::RelayLimits;

/// Default send timeout
// IF CHANGED, REMEMBER TO UPDATE THE DOCS!
pub const DEFAULT_SEND_TIMEOUT: Duration = Duration::from_secs(20);
pub(super) const DEFAULT_RETRY_SEC: u64 = 10;
pub(super) const MIN_RETRY_SEC: u64 = 5;
pub(super) const NEGENTROPY_HIGH_WATER_UP: usize = 100;
pub(super) const NEGENTROPY_LOW_WATER_UP: usize = 50;
pub(super) const NEGENTROPY_BATCH_SIZE_DOWN: usize = 50;

/// [`Relay`](super::Relay) options
#[derive(Debug, Clone)]
pub struct RelayOptions {
    pub(super) connection_mode: ConnectionMode,
    pub(super) flags: AtomicRelayServiceFlags,
    pow: Arc<AtomicU8>,
    reconnect: Arc<AtomicBool>,
    retry_sec: Arc<AtomicU64>,
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

    /// Set proxy
    #[deprecated(since = "0.33.0", note = "Use `connection_mode` instead")]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn proxy(mut self, proxy: Option<SocketAddr>) -> Self {
        match proxy {
            Some(proxy) => self.connection_mode = ConnectionMode::Proxy(proxy),
            None => self.connection_mode = ConnectionMode::Direct,
        };
        self
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
    #[deprecated(since = "0.35.0")]
    pub fn adjust_retry_sec(self, _adjust_retry_sec: bool) -> Self {
        self
    }

    /// Set adjust_retry_sec option
    #[deprecated(since = "0.35.0")]
    pub fn update_adjust_retry_sec(&self, _adjust_retry_sec: bool) {}

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
    pub(super) skip_disconnected: bool,
    pub(super) skip_send_confirmation: bool,
    pub(super) timeout: Duration,
}

impl Default for RelaySendOptions {
    fn default() -> Self {
        Self {
            skip_disconnected: true,
            skip_send_confirmation: false,
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
    pub fn skip_disconnected(mut self, value: bool) -> Self {
        self.skip_disconnected = value;
        self
    }

    /// Skip wait for confirmation that message is sent (default: false)
    pub fn skip_send_confirmation(mut self, value: bool) -> Self {
        self.skip_send_confirmation = value;
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
    pub(super) send_opts: RelaySendOptions,
}

impl SubscribeOptions {
    /// Set auto-close conditions
    pub fn close_on(mut self, opts: Option<SubscribeAutoCloseOptions>) -> Self {
        self.auto_close = opts;
        self
    }

    /// Set [RelaySendOptions]
    pub fn send_opts(mut self, opts: RelaySendOptions) -> Self {
        self.send_opts = opts;
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
pub enum NegentropyDirection {
    /// Send events to relay
    Up,
    /// Get events from relay
    #[default]
    Down,
    /// Both send and get events from relay (bidirectional sync)
    Both,
}

/// Negentropy reconciliation options
#[derive(Debug, Clone, Copy)]
pub struct NegentropyOptions {
    pub(super) initial_timeout: Duration,
    pub(super) direction: NegentropyDirection,
    pub(super) dry_run: bool,
}

impl Default for NegentropyOptions {
    fn default() -> Self {
        Self {
            initial_timeout: Duration::from_secs(10),
            direction: NegentropyDirection::default(),
            dry_run: false,
        }
    }
}

impl NegentropyOptions {
    /// New default [`NegentropyOptions`]
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
    pub fn direction(mut self, direction: NegentropyDirection) -> Self {
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

    #[inline]
    pub(super) fn do_up(&self) -> bool {
        !self.dry_run
            && matches!(
                self.direction,
                NegentropyDirection::Up | NegentropyDirection::Both
            )
    }

    #[inline]
    pub(super) fn do_down(&self) -> bool {
        !self.dry_run
            && matches!(
                self.direction,
                NegentropyDirection::Down | NegentropyDirection::Both
            )
    }
}
