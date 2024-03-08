// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::new_without_default)]

nostr_ffi::uniffi_reexport_scaffolding!();

mod client;
mod database;
mod error;
mod logger;
pub mod profile;
mod relay;

trait FromResult<T>: Sized {
    fn from_result(_: T) -> error::Result<Self>;
}

pub use crate::client::{Client, ClientBuilder, HandleNotification, Options};
pub use crate::database::NostrDatabase;
pub use crate::error::NostrSdkError;
pub use crate::logger::{init_logger, LogLevel};
pub use crate::relay::{Relay, RelayConnectionStats, RelayStatus};

uniffi::setup_scaffolding!("nostr_sdk");
