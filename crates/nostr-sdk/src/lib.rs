// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! High level Nostr client library.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)] // TODO: remove when MSRV >= 1.72.0, required for `clippy::arc_with_non_send_sync`
#![allow(clippy::arc_with_non_send_sync)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(feature = "all-nips", doc = include_str!("../README.md"))]

#[doc(hidden)]
pub use async_utility;
#[doc(hidden)]
pub use nostr::{self, *};
#[doc(hidden)]
pub use nostr_database::{self as database, NostrDatabase, NostrDatabaseExt};
#[doc(hidden)]
#[cfg(all(target_arch = "wasm32", feature = "indexeddb"))]
pub use nostr_indexeddb::{IndexedDBError, WebDatabase};
#[doc(hidden)]
#[cfg(feature = "ndb")]
pub use nostr_ndb::{self as ndb, NdbDatabase};
#[doc(hidden)]
pub use nostr_relay_pool::{
    self as pool, AtomicRelayServiceFlags, FilterOptions, NegentropyDirection, NegentropyOptions,
    Relay, RelayConnectionStats, RelayOptions, RelayPool, RelayPoolNotification, RelayPoolOptions,
    RelaySendOptions, RelayServiceFlags, RelayStatus, SubscribeAutoCloseOptions, SubscribeOptions,
};
#[doc(hidden)]
#[cfg(feature = "rocksdb")]
pub use nostr_rocksdb::RocksDatabase;
#[doc(hidden)]
pub use nostr_signer::{self as signer, NostrSigner, NostrSignerType};
#[doc(hidden)]
#[cfg(feature = "sqlite")]
pub use nostr_sqlite::{Error as SQLiteError, SQLiteDatabase};
#[doc(hidden)]
#[cfg(all(target_arch = "wasm32", feature = "webln"))]
pub use nostr_webln::WebLNZapper;
#[doc(hidden)]
#[cfg(feature = "nip57")]
pub use nostr_zapper::{self as zapper, NostrZapper, ZapperBackend, ZapperError};
#[doc(hidden)]
#[cfg(feature = "nip47")]
pub use nwc::{self, NostrWalletConnectOptions, NWC};

pub mod client;
pub mod prelude;

pub use self::client::{Client, ClientBuilder, Options};
