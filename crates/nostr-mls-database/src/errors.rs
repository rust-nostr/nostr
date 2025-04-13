// Copyright (c) 2025 Jeff Gardner
// Copyright (c) 2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr MLS database errors

use std::fmt;
/// Nostr MLS database error
#[derive(Debug, PartialEq)]
pub enum Error {}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
