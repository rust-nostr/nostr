// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NWC error

use std::fmt;

use nostr_sdk::{client, relay};

/// NWC error
#[derive(Debug)]
pub enum Error {
    /// Nostr protocol error
    Protocol(nostr::error::Error),
    /// Client error
    Client(client::Error),
    /// Relay error
    Relay(relay::Error),
    /// Response not received
    ResponseNotReceived,
    /// Request timeout
    Timeout,
    /// Handler error
    Handler(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::Client(e) => e.fmt(f),
            Self::Relay(e) => e.fmt(f),
            Self::ResponseNotReceived => f.write_str("response not received"),
            Self::Timeout => f.write_str("timeout"),
            Self::Handler(e) => f.write_str(e),
        }
    }
}

impl From<nostr::error::Error> for Error {
    fn from(e: nostr::error::Error) -> Self {
        Self::Protocol(e)
    }
}

impl From<client::Error> for Error {
    fn from(e: client::Error) -> Self {
        Self::Client(e)
    }
}

impl From<relay::Error> for Error {
    fn from(e: relay::Error) -> Self {
        Self::Relay(e)
    }
}
