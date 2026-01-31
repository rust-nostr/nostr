use std::fmt;

use nostr::RelayUrl;

use crate::relay;

/// Relay Pool error
#[derive(Debug)]
pub enum Error {
    /// Relay error
    Relay(relay::Error),
    /// Too many relays
    TooManyRelays {
        /// Max numer allowed
        limit: usize,
    },
    /// No relays specified
    NoRelaysSpecified,
    /// Relay not found
    RelayNotFound(RelayUrl),
    /// Relay Pool is shutdown
    Shutdown,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Relay(e) => e.fmt(f),
            Self::TooManyRelays { .. } => f.write_str("too many relays"),
            Self::NoRelaysSpecified => f.write_str("no relays specified"),
            Self::RelayNotFound(url) => write!(f, "relay '{}' not found", url),
            Self::Shutdown => f.write_str("relay pool is shutdown"),
        }
    }
}

impl From<relay::Error> for Error {
    fn from(e: relay::Error) -> Self {
        Self::Relay(e)
    }
}
