// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![allow(clippy::new_without_default)]

nostr_ffi::uniffi_reexport_scaffolding!();

pub mod client;
pub mod connect;
pub mod database;
pub mod error;
pub mod logger;
pub mod mock;
pub mod negentropy;
pub mod notifications;
pub mod nwc;
pub mod pool;
pub mod profile;
pub mod relay;

pub use self::client::{Client, ClientBuilder, Options};
pub use self::database::NostrDatabase;
pub use self::error::NostrSdkError;
pub use self::logger::{init_logger, LogLevel};
pub use self::notifications::HandleNotification;
pub use self::relay::{Relay, RelayConnectionStats, RelayStatus};

uniffi::setup_scaffolding!();
