// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Connect (NIP46)
//!
//! <https://github.com/nostr-protocol/nips/blob/master/46.md>

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]

pub mod client;
pub mod error;
pub mod prelude;
pub mod signer;
