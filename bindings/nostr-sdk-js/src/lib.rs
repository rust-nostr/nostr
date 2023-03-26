// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(clippy::drop_non_drop)]

use wasm_bindgen::prelude::*;

pub use nostr_js;

pub mod client;
//pub mod relay;

#[wasm_bindgen(js_name = initLogger)]
pub fn init_logger() {
    wasm_logger::init(wasm_logger::Config::default());
}
