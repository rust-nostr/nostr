// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(unknown_lints, clippy::arc_with_non_send_sync)]
#![allow(clippy::drop_non_drop)]

use nostr_js::error::{into_err, Result};
use wasm_bindgen::prelude::*;

pub use nostr_js;

pub mod client;
pub mod database;
pub mod profile;
//pub mod relay;

#[wasm_bindgen(js_name = initLogger)]
pub fn init_logger() -> Result<()> {
    tracing_wasm::try_set_as_global_default().map_err(into_err)
}
