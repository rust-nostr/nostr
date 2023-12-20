// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::net::AddrParseError;

use nostr::message::MessageHandleError;
use uniffi::Error;

pub type Result<T, E = NostrError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum NostrError {
    Generic { err: String },
}

impl std::error::Error for NostrError {}

impl fmt::Display for NostrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic { err } => write!(f, "{err}"),
        }
    }
}

impl From<std::num::ParseIntError> for NostrError {
    fn from(e: std::num::ParseIntError) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<std::char::ParseCharError> for NostrError {
    fn from(e: std::char::ParseCharError) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::key::Error> for NostrError {
    fn from(e: nostr::key::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::key::vanity::Error> for NostrError {
    fn from(e: nostr::key::vanity::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<MessageHandleError> for NostrError {
    fn from(e: MessageHandleError) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::types::metadata::Error> for NostrError {
    fn from(e: nostr::types::metadata::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::event::Error> for NostrError {
    fn from(e: nostr::event::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::event::builder::Error> for NostrError {
    fn from(e: nostr::event::builder::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::event::unsigned::Error> for NostrError {
    fn from(e: nostr::event::unsigned::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::event::tag::Error> for NostrError {
    fn from(e: nostr::event::tag::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip04::Error> for NostrError {
    fn from(e: nostr::nips::nip04::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip05::Error> for NostrError {
    fn from(e: nostr::nips::nip05::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip11::Error> for NostrError {
    fn from(e: nostr::nips::nip11::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip19::Error> for NostrError {
    fn from(e: nostr::nips::nip19::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip21::Error> for NostrError {
    fn from(e: nostr::nips::nip21::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip26::Error> for NostrError {
    fn from(e: nostr::nips::nip26::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip44::Error> for NostrError {
    fn from(e: nostr::nips::nip44::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip46::Error> for NostrError {
    fn from(e: nostr::nips::nip46::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip53::Error> for NostrError {
    fn from(e: nostr::nips::nip53::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::nips::nip90::Error> for NostrError {
    fn from(e: nostr::nips::nip90::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::secp256k1::Error> for NostrError {
    fn from(e: nostr::secp256k1::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::url::ParseError> for NostrError {
    fn from(e: nostr::url::ParseError) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::hashes::hex::Error> for NostrError {
    fn from(e: nostr::hashes::hex::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::serde_json::Error> for NostrError {
    fn from(e: nostr::serde_json::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::event::id::Error> for NostrError {
    fn from(e: nostr::event::id::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::types::channel_id::Error> for NostrError {
    fn from(e: nostr::types::channel_id::Error) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<AddrParseError> for NostrError {
    fn from(e: AddrParseError) -> NostrError {
        Self::Generic { err: e.to_string() }
    }
}
