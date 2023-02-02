// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![warn(missing_docs)]

//! High level Nostr client library.

#[cfg(feature = "blocking")]
use once_cell::sync::Lazy;
#[cfg(feature = "blocking")]
use tokio::runtime::Runtime;

pub use nostr;
pub use nostr::Result;

pub mod client;
#[cfg(feature = "all-nips")]
mod doctest;
pub mod prelude;
pub mod relay;
pub mod subscription;
mod thread;

#[cfg(feature = "blocking")]
pub use self::client::blocking;
pub use self::client::{Client, Options};
pub use self::relay::pool::{RelayPool, RelayPoolNotification};
pub use self::relay::{Relay, RelayOptions, RelayStatus};

#[cfg(feature = "blocking")]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Can't start Tokio runtime"));
