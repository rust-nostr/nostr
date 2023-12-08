// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use wasm_bindgen::JsValue;

pub type Result<T, E = JsValue> = std::result::Result<T, E>;

/// Helper to replace the `E` to `Error` to `napi::Error` conversion.
#[inline]
pub fn into_err<E>(error: E) -> JsValue
where
    E: std::error::Error,
{
    JsValue::from_str(&error.to_string())
}
