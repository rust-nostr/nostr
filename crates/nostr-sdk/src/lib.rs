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

#[cfg(all(target_arch = "wasm32", feature = "sqlite"))]
compile_error!("`sqlite` feature can't be enabled for WASM targets");

#[cfg(feature = "blocking")]
use once_cell::sync::Lazy;
#[cfg(feature = "blocking")]
use tokio::runtime::Runtime;

pub use nostr::{self, *};

pub mod client;
pub mod prelude;
pub mod relay;

#[cfg(feature = "blocking")]
pub use self::client::blocking;
pub use self::client::Client;
#[cfg(not(target_arch = "wasm32"))]
pub use self::client::Options;
pub use self::relay::{RelayOptions, RelayPoolNotification, RelayStatus};

#[cfg(feature = "blocking")]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Can't start Tokio runtime"));
