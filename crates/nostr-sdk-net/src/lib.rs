// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Network

pub extern crate futures_util;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::Message as WsMessage;
#[cfg(target_arch = "wasm32")]
pub use ws_stream_wasm::WsMessage;
