// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::new_without_default)]

pub mod client;
pub mod connect;
pub mod database;
pub mod error;
pub mod logger;
pub mod negentropy;
pub mod notifications;
pub mod nwc;
pub mod protocol;
pub mod relay;
mod util;

/// Get git hash version of `rust-nostr` libraries
#[uniffi::export]
pub fn git_hash_version() -> Option<String> {
    option_env!("GIT_HASH").map(|v| v.to_string())
}

// Workaround to fix UPX compression error
//
// Error: CantPackException: need DT_INIT; try "void _init(void){}"
// Workaround comes from https://github.com/upx/upx/issues/740
#[no_mangle]
#[cfg(target_os = "android")]
pub fn _init() {}

// Changes to this arg will break binding packages (in particular Swift).
// If this is removed, make sure to update `uniffi.toml`
uniffi::setup_scaffolding!("nostr_sdk");
