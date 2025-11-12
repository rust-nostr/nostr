// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr SQL database

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(not(any(feature = "sqlite", feature = "postgres", feature = "mysql")))]
compile_error!("At least one database backend must be enabled");

pub mod db;
pub mod error;
mod model;
// #[cfg(feature = "sqlite")]
// pub mod sqlite;

pub use self::db::{NostrSql, NostrSqlBackend};
