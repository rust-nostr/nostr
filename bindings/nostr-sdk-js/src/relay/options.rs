// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::flags::JsRelayServiceFlags;
use super::limits::JsRelayLimits;
use crate::duration::JsDuration;

/// `Relay` options
#[wasm_bindgen(js_name = RelayOptions)]
pub struct JsRelayOptions {
    inner: RelayOptions,
}

impl Deref for JsRelayOptions {
    type Target = RelayOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<RelayOptions> for JsRelayOptions {
    fn from(inner: RelayOptions) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = RelayOptions)]
impl JsRelayOptions {
    /// New default relay options
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RelayOptions::new(),
        }
    }

    /// Set Relay Service Flags
    pub fn flags(self, flags: &JsRelayServiceFlags) -> Self {
        self.inner.flags(**flags).into()
    }

    /// Set read flag
    pub fn read(self, read: bool) -> Self {
        self.inner.read(read).into()
    }

    /// Set write flag
    pub fn write(self, write: bool) -> Self {
        self.inner.write(write).into()
    }

    /// Set ping flag
    pub fn ping(self, ping: bool) -> Self {
        self.inner.ping(ping).into()
    }

    /// Enable/disable auto reconnection (default: true)
    pub fn reconnect(self, reconnect: bool) -> Self {
        self.inner.reconnect(reconnect).into()
    }

    /// Retry connection time (default: 10 sec)
    pub fn retry_interval(self, interval: &JsDuration) -> Self {
        self.inner.retry_interval(**interval).into()
    }

    /// Automatically adjust retry interval based on success/attempts (default: true)
    pub fn adjust_retry_interval(self, adjust_retry_interval: bool) -> Self {
        self.inner
            .adjust_retry_interval(adjust_retry_interval)
            .into()
    }

    /// Set custom limits
    pub fn limits(self, limits: &JsRelayLimits) -> Self {
        self.inner.limits(limits.deref().clone()).into()
    }
}

/// Request (REQ) exit policy
#[wasm_bindgen(js_name = ReqExitPolicy)]
pub struct JsReqExitPolicy {
    inner: ReqExitPolicy,
}

impl Deref for JsReqExitPolicy {
    type Target = ReqExitPolicy;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[wasm_bindgen(js_class = ReqExitPolicy)]
impl JsReqExitPolicy {
    /// Exit on EOSE
    #[wasm_bindgen(js_name = exitOnEose)]
    pub fn exit_on_eose() -> Self {
        Self {
            inner: ReqExitPolicy::ExitOnEOSE,
        }
    }

    /// After EOSE is received, keep listening for N more events that match the filter
    #[wasm_bindgen(js_name = waitForEventsAfterEOSE)]
    pub fn wait_for_events_after_eose(num: u16) -> Self {
        Self {
            inner: ReqExitPolicy::WaitForEventsAfterEOSE(num),
        }
    }

    /// After EOSE is received, keep listening for matching events for `Duration` more time
    #[wasm_bindgen(js_name = waitDurationAfterEOSE)]
    pub fn wait_duration_after_eose(duration: &JsDuration) -> Self {
        Self {
            inner: ReqExitPolicy::WaitDurationAfterEOSE(**duration),
        }
    }
}

/// Auto-closing subscribe options
#[wasm_bindgen(js_name = SubscribeAutoCloseOptions)]
pub struct JsSubscribeAutoCloseOptions {
    inner: SubscribeAutoCloseOptions,
}

impl Deref for JsSubscribeAutoCloseOptions {
    type Target = SubscribeAutoCloseOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<SubscribeAutoCloseOptions> for JsSubscribeAutoCloseOptions {
    fn from(inner: SubscribeAutoCloseOptions) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = SubscribeAutoCloseOptions)]
impl JsSubscribeAutoCloseOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SubscribeAutoCloseOptions::default(),
        }
    }

    /// Close subscription when the policy is satisfied
    #[wasm_bindgen(js_name = exitPolicy)]
    pub fn exit_policy(self, policy: JsReqExitPolicy) -> Self {
        self.inner.exit_policy(policy.inner).into()
    }

    /// Automatically close subscription after `Duration`
    pub fn timeout(self, timeout: Option<JsDuration>) -> Self {
        self.inner.timeout(timeout.map(|t| *t)).into()
    }

    /// Automatically close subscription if no notifications/events are received within the duration.
    #[wasm_bindgen(js_name = idleTimeout)]
    pub fn idle_timeout(self, timeout: Option<JsDuration>) -> Self {
        self.inner.idle_timeout(timeout.map(|t| *t)).into()
    }
}

/// Subscribe options
#[wasm_bindgen(js_name = SubscribeOptions)]
pub struct JsSubscribeOptions {
    inner: SubscribeOptions,
}

impl Deref for JsSubscribeOptions {
    type Target = SubscribeOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<SubscribeOptions> for JsSubscribeOptions {
    fn from(inner: SubscribeOptions) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = SubscribeOptions)]
impl JsSubscribeOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SubscribeOptions::default(),
        }
    }

    /// Set auto-close conditions
    pub fn close_on(self, opts: Option<JsSubscribeAutoCloseOptions>) -> Self {
        self.inner.close_on(opts.map(|o| *o)).into()
    }
}

#[wasm_bindgen(js_name = SyncDirection)]
pub enum JsSyncDirection {
    Up,
    Down,
    Both,
}

impl From<JsSyncDirection> for SyncDirection {
    fn from(value: JsSyncDirection) -> Self {
        match value {
            JsSyncDirection::Up => Self::Up,
            JsSyncDirection::Down => Self::Down,
            JsSyncDirection::Both => Self::Both,
        }
    }
}

#[wasm_bindgen(js_name = SyncOptions)]
pub struct JsSyncOptions {
    inner: SyncOptions,
}

impl Deref for JsSyncOptions {
    type Target = SyncOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<SyncOptions> for JsSyncOptions {
    fn from(inner: SyncOptions) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = SyncOptions)]
impl JsSyncOptions {
    /// New default options
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SyncOptions::new(),
        }
    }

    /// Timeout to check if negentropy it's supported (default: 10 secs)
    #[wasm_bindgen(js_name = initialTimeout)]
    pub fn initial_timeout(self, timeout: JsDuration) -> Self {
        self.inner.initial_timeout(*timeout).into()
    }

    /// Sync direction (default: down)
    pub fn direction(self, direction: JsSyncDirection) -> Self {
        self.inner.direction(direction.into()).into()
    }

    /// Dry run
    ///
    /// Just check what event are missing: execute reconciliation but WITHOUT
    /// getting/sending full events.
    pub fn dry_run(self) -> Self {
        self.inner.dry_run().into()
    }
}
