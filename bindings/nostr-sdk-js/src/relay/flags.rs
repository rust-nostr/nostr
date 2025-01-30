// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

/// Flag checks
#[wasm_bindgen(js_name = FlagCheck)]
pub enum JsFlagCheck {
    /// Use `OR` logic operator
    Any,
    /// Use `AND` logic operator
    All,
}

impl From<JsFlagCheck> for FlagCheck {
    fn from(value: JsFlagCheck) -> Self {
        match value {
            JsFlagCheck::Any => Self::Any,
            JsFlagCheck::All => Self::All,
        }
    }
}

#[wasm_bindgen(js_name = RelayServiceFlags)]
pub struct JsRelayServiceFlags {
    inner: RelayServiceFlags,
}

impl Deref for JsRelayServiceFlags {
    type Target = RelayServiceFlags;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<RelayServiceFlags> for JsRelayServiceFlags {
    fn from(inner: RelayServiceFlags) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = RelayServiceFlags)]
impl JsRelayServiceFlags {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RelayServiceFlags::NONE,
        }
    }

    /// Default flags: `READ`, `WRITE` and `PING`
    pub fn default() -> Self {
        Self {
            inner: RelayServiceFlags::default(),
        }
    }

    pub fn read() -> Self {
        Self {
            inner: RelayServiceFlags::READ,
        }
    }

    pub fn write() -> Self {
        Self {
            inner: RelayServiceFlags::WRITE,
        }
    }

    pub fn ping() -> Self {
        Self {
            inner: RelayServiceFlags::PING,
        }
    }

    /// Add `RelayServiceFlags` together.
    pub fn add(&mut self, other: &JsRelayServiceFlags) {
        self.inner.add(**other);
    }

    /// Remove `RelayServiceFlags` from this.
    pub fn remove(&mut self, other: &JsRelayServiceFlags) {
        self.inner.remove(**other);
    }
}

#[wasm_bindgen(js_name = AtomicRelayServiceFlags)]
pub struct JsAtomicRelayServiceFlags {
    inner: AtomicRelayServiceFlags,
}

impl From<AtomicRelayServiceFlags> for JsAtomicRelayServiceFlags {
    fn from(inner: AtomicRelayServiceFlags) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = AtomicRelayServiceFlags)]
impl JsAtomicRelayServiceFlags {
    pub fn new(flags: &JsRelayServiceFlags) -> Self {
        Self {
            inner: AtomicRelayServiceFlags::new(**flags),
        }
    }

    pub fn add(&self, flags: &JsRelayServiceFlags) {
        self.inner.add(**flags);
    }

    pub fn remove(&self, flags: &JsRelayServiceFlags) {
        self.inner.remove(**flags);
    }

    /// Check whether `RelayServiceFlags` are included in this one.
    pub fn has(&self, flags: &JsRelayServiceFlags, check: JsFlagCheck) -> bool {
        self.inner.has(**flags, check.into())
    }

    /// Check if `READ` service is enabled
    pub fn has_read(&self) -> bool {
        self.inner.has_read()
    }

    /// Check if `WRITE` service is enabled
    pub fn has_write(&self) -> bool {
        self.inner.has_write()
    }

    /// Check if `PING` service is enabled
    pub fn has_ping(&self) -> bool {
        self.inner.has_ping()
    }
}
