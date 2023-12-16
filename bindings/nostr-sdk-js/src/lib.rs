// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::drop_non_drop)]
// TODO: remove when `per-package-target` feature will be stable
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

pub use nostr_js;

pub mod client;
//pub mod relay;

#[wasm_bindgen(js_name = initLogger)]
pub fn init_logger() {
    tracing_wasm::set_as_global_default();
}
