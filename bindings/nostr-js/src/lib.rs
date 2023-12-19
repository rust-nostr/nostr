// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::drop_non_drop)]
#![allow(non_snake_case)]

use wasm_bindgen::prelude::*;

pub mod error;
mod event;
mod key;
mod message;
pub mod nips;
mod types;

pub use self::event::{JsEvent, JsEventBuilder, JsEventId};
pub use self::key::{JsKeys, JsPublicKey, JsSecretKey};
pub use self::message::{JsFilter, JsSubscriptionId};
pub use self::types::{JsContact, JsMetadata};

/// Run some stuff when the Wasm module is instantiated.
///
/// Right now, it does the following:
///
/// * Redirect Rust panics to JavaScript console.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
