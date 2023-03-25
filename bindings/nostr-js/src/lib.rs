// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use wasm_bindgen::prelude::*;

mod error;
pub mod key;
// pub mod event;
// pub mod message;
// pub mod nips;
// pub mod types;

/// Run some stuff when the Wasm module is instantiated.
///
/// Right now, it does the following:
///
/// * Redirect Rust panics to JavaScript console.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}