// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(clippy::drop_non_drop)]

use wasm_bindgen::prelude::*;

pub mod error;
mod event;
mod future;
mod key;
mod message;
pub mod nips;
mod types;
pub mod util;

pub use self::event::{JsEvent, JsEventBuilder, JsEventId};
pub use self::key::{JsKeys, JsPublicKey, JsSecretKey};
pub use self::message::{JsFilter, JsSubscriptionId};
pub use self::types::{JsChannelId, JsContact, JsMetadata};

/// Run some stuff when the Wasm module is instantiated.
///
/// Right now, it does the following:
///
/// * Redirect Rust panics to JavaScript console.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
