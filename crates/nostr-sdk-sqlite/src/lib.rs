// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![warn(missing_docs)]

//! SQLite for Nostr SDK

mod migration;
pub mod store;

pub use self::store::{Error, Store};
