// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use wasm_bindgen::JsValue;

pub type Result<T, E = JsValue> = core::result::Result<T, E>;

pub fn into_err<E>(error: E) -> JsValue
where
    E: std::error::Error,
{
    JsValue::from_str(&error.to_string())
}

#[derive(Debug)]
pub(crate) struct MiddleError(String);

impl std::error::Error for MiddleError {}

impl fmt::Display for MiddleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<JsValue> for MiddleError {
    fn from(e: JsValue) -> Self {
        Self(format!("{e:?}"))
    }
}
