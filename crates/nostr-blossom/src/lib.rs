// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! This crate implements the Blossom protocol for decentralized content storage and retrieval.
//!
//! The Blossom protocol defines a standard for storing and retrieving blobs (binary large objects)
//! in a decentralized manner, using the Nostr protocol for authorization and discovery.
//!
//! <https://github.com/hzrd149/blossom>

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]

pub mod bud01;
pub mod bud02;
pub mod client;
pub mod prelude;
