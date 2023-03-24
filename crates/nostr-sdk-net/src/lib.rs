// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Network

pub extern crate futures_util;

pub mod error;
pub mod message;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use self::message::*;
