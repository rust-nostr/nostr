// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(
    all(feature = "all-nips", feature = "websocket"),
    doc = include_str!("../README.md")
)]

//! High level Nostr client library.

#[cfg(all(feature = "websocket", feature = "rest-api"))]
compile_error!("Feature 'websocket' and 'rest-api' can't be enabled at the same time");

#[cfg(all(feature = "websocket", feature = "blocking"))]
use once_cell::sync::Lazy;
#[cfg(all(feature = "websocket", feature = "blocking"))]
use tokio::runtime::Runtime;

pub use nostr::{self, *};

pub mod client;
pub mod prelude;
#[cfg(feature = "websocket")]
pub mod relay;

#[cfg(feature = "rest-api")]
pub use self::client::rest::Client;
#[cfg(feature = "websocket")]
pub use self::client::websocket::{Client, Options};
#[cfg(feature = "websocket")]
pub use self::relay::{Relay, RelayOptions, RelayPoolNotification, RelayStatus};

#[cfg(all(feature = "websocket", feature = "blocking"))]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Can't start Tokio runtime"));
