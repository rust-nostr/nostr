// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![doc = include_str!("../README.md")]

#[cfg(feature = "blocking")]
use once_cell::sync::Lazy;
#[cfg(feature = "blocking")]
use tokio::runtime::Runtime;

pub use nostr;
pub use nostr::Result;

pub mod client;
pub mod relay;
pub mod subscription;
mod thread;

#[cfg(feature = "blocking")]
pub use self::client::blocking;
pub use self::client::Client;
pub use self::relay::pool::{RelayPool, RelayPoolNotification};
pub use self::relay::{Relay, RelayStatus};

#[cfg(feature = "blocking")]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Can't start Tokio runtime"));
