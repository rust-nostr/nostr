// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

/// Event Kind
#[wasm_bindgen(js_name = Kind)]
pub struct JsKind {
    inner: Kind,
}

impl Deref for JsKind {
    type Target = Kind;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Kind> for JsKind {
    fn from(inner: Kind) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Kind)]
impl JsKind {
    #[wasm_bindgen(constructor)]
    pub fn new(kind: u16) -> Self {
        Self {
            inner: Kind::from_u16(kind),
        }
    }

    /// Get as 16-bit unsigned integer
    #[wasm_bindgen(js_name = asU16)]
    pub fn as_u16(&self) -> u16 {
        self.inner.as_u16()
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn _to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Check if it's regular
    ///
    /// Regular means that event is expected to be stored by relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isRegular)]
    pub fn is_regular(&self) -> bool {
        self.inner.is_regular()
    }

    /// Check if it's replaceable
    ///
    /// Replaceable means that, for each combination of `pubkey` and `kind`,
    /// only the latest event MUST be stored by relays, older versions MAY be discarded.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isReplaceable)]
    pub fn is_replaceable(&self) -> bool {
        self.inner.is_replaceable()
    }

    /// Check if it's ephemeral
    ///
    /// Ephemeral means that event is not expected to be stored by relays.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isEphemeral)]
    pub fn is_ephemeral(&self) -> bool {
        self.inner.is_ephemeral()
    }

    /// Check if it's parameterized replaceable
    ///
    /// Parametrized replaceable means that, for each combination of `pubkey`, `kind` and the `d` tag's first value,
    /// only the latest event MUST be stored by relays, older versions MAY be discarded.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[wasm_bindgen(js_name = isParametrizedReplaceable)]
    pub fn is_parameterized_replaceable(&self) -> bool {
        self.inner.is_parameterized_replaceable()
    }

    /// Check if it's a NIP-90 job request
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = isJobRequest)]
    pub fn is_job_request(&self) -> bool {
        self.inner.is_job_request()
    }

    /// Check if it's a NIP-90 job result
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/90.md>
    #[wasm_bindgen(js_name = isJobResult)]
    pub fn is_job_result(&self) -> bool {
        self.inner.is_job_result()
    }
}
