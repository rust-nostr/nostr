//! Nostr SQLite database

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod error;
mod migration;
mod model;
mod pool;
pub mod prelude;
pub mod store;
