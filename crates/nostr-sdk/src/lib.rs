// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)]
#![allow(clippy::arc_with_non_send_sync)]

//! High level Nostr client library.

#![cfg_attr(
    feature = "all-nips",
    doc = include_str!("../README.md")
)]

pub use async_utility;
pub use nostr::{self, *};
pub use nostr_database::{self as database, NostrDatabase, NostrDatabaseExt, Profile};
#[cfg(all(target_arch = "wasm32", feature = "indexeddb"))]
pub use nostr_indexeddb::{IndexedDBError, WebDatabase};
pub use nostr_relay_pool::{
    self as pool, AtomicRelayServiceFlags, FilterOptions, NegentropyDirection, NegentropyOptions,
    Relay, RelayConnectionStats, RelayOptions, RelayPool, RelayPoolNotification, RelayPoolOptions,
    RelaySendOptions, RelayServiceFlags, RelayStatus, SubscribeAutoCloseOptions, SubscribeOptions,
};
#[cfg(feature = "rocksdb")]
pub use nostr_rocksdb::RocksDatabase;
pub use nostr_signer::{self as signer, NostrSigner, NostrSignerType};
#[cfg(feature = "sqlite")]
pub use nostr_sqlite::{Error as SQLiteError, SQLiteDatabase};
#[cfg(all(target_arch = "wasm32", feature = "webln"))]
pub use nostr_webln::WebLNZapper;
#[cfg(feature = "nip57")]
pub use nostr_zapper::{self as zapper, NostrZapper, ZapperBackend, ZapperError};
#[cfg(feature = "nip47")]
pub use nwc::{self, NostrWalletConnectOptions, NWC};

pub mod client;
pub mod prelude;

pub use self::client::{Client, ClientBuilder, Options};
