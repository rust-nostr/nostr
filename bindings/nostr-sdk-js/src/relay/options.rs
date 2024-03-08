// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use nostr_sdk::{
    FilterOptions, NegentropyDirection, NegentropyOptions, SubscribeAutoCloseOptions,
    SubscribeOptions,
};
use wasm_bindgen::prelude::*;

use crate::duration::JsDuration;

/// Filter options
#[wasm_bindgen(js_name = FilterOptions)]
pub struct JsFilterOptions {
    inner: FilterOptions,
}

#[wasm_bindgen(js_class = FilterOptions)]
impl JsFilterOptions {
    /// Exit on EOSE
    #[wasm_bindgen(js_name = exitOnEose)]
    pub fn exit_on_eose() -> Self {
        Self {
            inner: FilterOptions::ExitOnEOSE,
        }
    }

    /// After EOSE is received, keep listening for N more events that match the filter, then return
    #[wasm_bindgen(js_name = waitForEventsAfterEOSE)]
    pub fn wait_for_events_after_eose(num: u16) -> Self {
        Self {
            inner: FilterOptions::WaitForEventsAfterEOSE(num),
        }
    }

    /// After EOSE is received, keep listening for matching events for `Duration` more time, then return
    #[wasm_bindgen(js_name = waitDurationAfterEOSE)]
    pub fn wait_duration_after_eose(duration: &JsDuration) -> Self {
        Self {
            inner: FilterOptions::WaitDurationAfterEOSE(**duration),
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

    /// Close subscription when `FilterOptions` is satisfied
    pub fn filter(self, filter: JsFilterOptions) -> Self {
        self.inner.filter(filter.inner).into()
    }

    /// Automatically close subscription after `Duration`
    pub fn timeout(self, timeout: Option<JsDuration>) -> Self {
        self.inner.timeout(timeout.map(|t| *t)).into()
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

    /* /// Set [RelaySendOptions]
    pub fn send_opts(self, opts: JsRelaySendOptions) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.send_opts(**opts);
        builder
    } */
}

#[wasm_bindgen(js_name = NegentropyDirection)]
pub enum JsNegentropyDirection {
    Up,
    Down,
    Both,
}

impl From<JsNegentropyDirection> for NegentropyDirection {
    fn from(value: JsNegentropyDirection) -> Self {
        match value {
            JsNegentropyDirection::Up => Self::Up,
            JsNegentropyDirection::Down => Self::Down,
            JsNegentropyDirection::Both => Self::Both,
        }
    }
}

#[wasm_bindgen(js_name = NegentropyOptions)]
pub struct JsNegentropyOptions {
    inner: NegentropyOptions,
}

impl Deref for JsNegentropyOptions {
    type Target = NegentropyOptions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NegentropyOptions> for JsNegentropyOptions {
    fn from(inner: NegentropyOptions) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = NegentropyOptions)]
impl JsNegentropyOptions {
    /// New default options
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: NegentropyOptions::new(),
        }
    }

    /// Timeout to check if negentropy it's supported (default: 10 secs)
    #[wasm_bindgen(js_name = initialTimeout)]
    pub fn initial_timeout(self, timeout: JsDuration) -> Self {
        self.inner.initial_timeout(*timeout).into()
    }

    /// Negentropy Sync direction (default: down)
    pub fn direction(self, direction: JsNegentropyDirection) -> Self {
        self.inner.direction(direction.into()).into()
    }
}
