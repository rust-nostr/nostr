// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! High level Nostr client library.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]
#![allow(unknown_lints)] // TODO: remove when MSRV >= 1.72.0, required for `clippy::arc_with_non_send_sync`
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::mutable_key_type)] // TODO: remove when possible. Needed to suppress false positive for `BTreeSet<Event>`
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(feature = "all-nips", doc = include_str!("../README.md"))]

#[doc(hidden)]
pub use async_utility;
#[doc(hidden)]
pub use nostr::{self, *};
#[doc(hidden)]
#[cfg(all(target_arch = "wasm32", feature = "indexeddb"))]
pub use nostr_indexeddb::WebDatabase;
#[doc(hidden)]
#[cfg(feature = "lmdb")]
pub use nostr_lmdb::NostrLMDB;
#[doc(hidden)]
#[cfg(feature = "ndb")]
pub use nostr_ndb::{self as ndb, NdbDatabase};
#[doc(hidden)]
pub use nostr_relay_pool::{
    self as pool, AtomicRelayServiceFlags, Relay, RelayConnectionStats, RelayOptions, RelayPool,
    RelayPoolNotification, RelayPoolOptions, RelayServiceFlags, RelayStatus,
    SubscribeAutoCloseOptions, SubscribeOptions, SyncDirection, SyncOptions,
};
#[doc(hidden)]
#[cfg(feature = "nip57")]
pub use nostr_zapper::{self as zapper, *};

pub mod client;
mod gossip;
pub mod prelude;

pub use self::client::{Client, ClientBuilder, Options};
