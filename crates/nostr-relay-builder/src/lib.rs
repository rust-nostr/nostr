// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Relay Builder and Mock Relay for tests

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]

pub mod builder;
pub mod error;
pub mod local;
pub mod mock;
pub mod prelude;

pub use self::builder::RelayBuilder;
pub use self::error::Error;
pub use self::local::LocalRelay;
pub use self::mock::MockRelay;
