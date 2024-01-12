// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr SDK Network

#![forbid(unsafe_code)]
#![deny(warnings)]

pub extern crate futures_util;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm_ws::WsMessage;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::Message as WsMessage;
