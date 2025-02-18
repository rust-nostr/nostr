// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_without_default)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::drop_non_drop)]
#![allow(non_snake_case)]
// rust-analyzer not work well with multiple different targets in workspace
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

pub mod abortable;
pub mod client;
pub mod connect;
pub mod database;
pub mod duration;
pub mod error;
pub mod logger;
pub mod nwc;
pub mod policy;
pub mod protocol;
pub mod relay;
pub mod signer;
mod util;

#[wasm_bindgen]
extern "C" {
    /// String array
    #[wasm_bindgen(typescript_type = "string[]")]
    pub type JsStringArray;
}

/// Run some stuff when the Wasm module is instantiated.
///
/// Right now, it does the following:
///
/// * Redirect Rust panics to JavaScript console.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = gitHashVersion)]
pub fn git_hash_version() -> Option<String> {
    option_env!("GIT_HASH").map(|v| v.to_string())
}
