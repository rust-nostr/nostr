// Copyright (c) 2022-2023 Yuki Kishimoto
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
    wasm_logger::init(wasm_logger::Config::default());
}
