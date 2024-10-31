// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::sync::LazyLock;

use tokio::runtime::Runtime;

pub mod client;
pub mod logger;
pub mod protocol;

static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("failed to create tokio runtime"));

pub fn git_hash_version() -> String {
    option_env!("GIT_HASH").unwrap_or_default().to_owned()
}

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn git_hash_version() -> String;
    }
}
