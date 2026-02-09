// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Gossip SQLite store.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]

mod constant;
pub mod error;
mod migration;
mod model;
mod pool;
pub mod prelude;
pub mod store;
