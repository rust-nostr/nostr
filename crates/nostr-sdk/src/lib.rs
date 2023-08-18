// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

//! High level Nostr client library.

#![cfg_attr(
    feature = "all-nips",
    doc = include_str!("../README.md")
)]

#[cfg(all(target_arch = "wasm32", feature = "blocking"))]
compile_error!("`blocking` feature can't be enabled for WASM targets");

pub use nostr::{self, *};
#[cfg(feature = "blocking")]
use nostr_sdk_net::futures_util::Future;
#[cfg(feature = "blocking")]
use once_cell::sync::Lazy;
#[cfg(feature = "blocking")]
use tokio::runtime::Runtime;

pub mod client;
pub mod prelude;
pub mod relay;
pub mod util;

#[cfg(feature = "blocking")]
pub use self::client::blocking;
pub use self::client::{Client, Options};
pub use self::relay::{
    ActiveSubscription, FilterOptions, InternalSubscriptionId, Relay, RelayConnectionStats,
    RelayOptions, RelayPoolNotification, RelayPoolOptions, RelaySendOptions, RelayStatus,
};

#[cfg(feature = "blocking")]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Can't start Tokio runtime"));

#[allow(missing_docs)]
#[cfg(feature = "blocking")]
pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}
