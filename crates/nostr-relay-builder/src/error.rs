// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Relay builder error

use std::io;

#[cfg(feature = "tor")]
use async_wsocket::native::tor;
use thiserror::Error;

/// Relay builder error
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error
    #[error(transparent)]
    IO(#[from] io::Error),
    /// Tor error
    #[cfg(feature = "tor")]
    #[error(transparent)]
    Tor(#[from] tor::Error),
    /// No port available
    #[error("No port available")]
    NoPortAvailable,
}
