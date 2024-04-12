// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_without_default)]
#![allow(clippy::drop_non_drop)]
#![allow(non_snake_case)]
// rust-analyzer not work well with multiple different targets in workspace
#![cfg(target_arch = "wasm32")]

pub use nostr_js;

pub mod abortable;
pub mod client;
pub mod database;
pub mod duration;
pub mod logger;
pub mod nwc;
pub mod profile;
pub mod relay;
