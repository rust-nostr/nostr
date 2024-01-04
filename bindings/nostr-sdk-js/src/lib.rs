// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(unknown_lints, clippy::arc_with_non_send_sync)]
#![allow(clippy::new_without_default)]
#![allow(clippy::drop_non_drop)]

pub use nostr_js;

pub mod client;
pub mod database;
pub mod logger;
pub mod profile;
