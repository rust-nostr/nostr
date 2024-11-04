// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ffi::{c_char, CString};
use std::sync::LazyLock;

use tokio::runtime::Runtime;

pub mod client;
pub mod error;
pub mod logger;
pub mod protocol;

static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("failed to create tokio runtime"));

#[inline]
fn get_git_hash() -> &'static str {
    option_env!("GIT_HASH").unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn git_hash_version() -> *const c_char {
    let hash: &str = get_git_hash();
    let c_string = CString::new(hash).unwrap();
    c_string.into_raw()
}
