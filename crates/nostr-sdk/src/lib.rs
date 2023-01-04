// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#![doc = include_str!("../README.md")]

#[cfg(feature = "blocking")]
use once_cell::sync::Lazy;
#[cfg(feature = "blocking")]
use tokio::runtime::{Builder, Runtime};

pub use nostr;
pub use nostr::Result;
#[cfg(feature = "sqlite")]
pub use nostr_sdk_sqlite as sqlite;

pub mod client;
pub mod relay;
pub mod subscription;

#[cfg(feature = "blocking")]
pub use self::client::blocking;
pub use self::client::Client;
pub use self::relay::pool::{RelayPool, RelayPoolNotifications};
pub use self::relay::{Relay, RelayStatus};

#[cfg(feature = "blocking")]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Can't start Tokio runtime"));

#[cfg(feature = "blocking")]
fn new_current_thread() -> nostr::Result<Runtime> {
    Ok(Builder::new_current_thread().enable_all().build()?)
}
