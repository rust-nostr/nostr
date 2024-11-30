// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::new_without_default)]

use uniffi::Object;

pub mod client;
pub mod connect;
pub mod database;
pub mod error;
pub mod logger;
pub mod negentropy;
pub mod notifications;
pub mod nwc;
pub mod pool;
pub mod profile;
pub mod protocol;
pub mod relay;
mod util;

#[derive(Object)]
pub struct NostrLibrary;

#[uniffi::export]
impl NostrLibrary {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn git_hash_version(&self) -> Option<String> {
        option_env!("GIT_HASH").map(|v| v.to_string())
    }
}

// Changes to this arg will break binding packages (in particular Swift).
// If this is removed, make sure to update `uniffi.toml`
uniffi::setup_scaffolding!("nostr_sdk");
