// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::net::AddrParseError;

use nostr::message::MessageHandleError;
use tracing::subscriber::SetGlobalDefaultError;
use uniffi::Error;

pub type Result<T, E = NostrSdkError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[uniffi(flat_error)]
pub enum NostrSdkError {
    Generic(String),
}

impl std::error::Error for NostrSdkError {}

impl fmt::Display for NostrSdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(e) => write!(f, "{e}"),
        }
    }
}

impl From<std::num::ParseIntError> for NostrSdkError {
    fn from(e: std::num::ParseIntError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<std::char::ParseCharError> for NostrSdkError {
    fn from(e: std::char::ParseCharError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::signer::SignerError> for NostrSdkError {
    fn from(e: nostr::signer::SignerError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::key::Error> for NostrSdkError {
    fn from(e: nostr::key::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::key::vanity::Error> for NostrSdkError {
    fn from(e: nostr::key::vanity::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<MessageHandleError> for NostrSdkError {
    fn from(e: MessageHandleError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::types::metadata::Error> for NostrSdkError {
    fn from(e: nostr::types::metadata::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::event::Error> for NostrSdkError {
    fn from(e: nostr::event::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::event::builder::Error> for NostrSdkError {
    fn from(e: nostr::event::builder::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::event::unsigned::Error> for NostrSdkError {
    fn from(e: nostr::event::unsigned::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::event::tag::Error> for NostrSdkError {
    fn from(e: nostr::event::tag::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip01::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip01::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip04::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip04::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip05::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip05::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip06::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip06::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip11::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip11::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip19::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip19::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip21::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip21::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip26::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip26::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip44::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip44::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip46::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip46::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip47::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip47::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip49::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip49::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip53::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip53::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip57::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip57::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip59::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip59::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::nips::nip90::Error> for NostrSdkError {
    fn from(e: nostr::nips::nip90::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::secp256k1::Error> for NostrSdkError {
    fn from(e: nostr::secp256k1::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::types::url::ParseError> for NostrSdkError {
    fn from(e: nostr::types::url::ParseError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::hashes::hex::HexToArrayError> for NostrSdkError {
    fn from(e: nostr::hashes::hex::HexToArrayError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::serde_json::Error> for NostrSdkError {
    fn from(e: nostr::serde_json::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr::event::id::Error> for NostrSdkError {
    fn from(e: nostr::event::id::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<AddrParseError> for NostrSdkError {
    fn from(e: AddrParseError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<SetGlobalDefaultError> for NostrSdkError {
    fn from(e: SetGlobalDefaultError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::client::Error> for NostrSdkError {
    fn from(e: nostr_sdk::client::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::pool::relay::Error> for NostrSdkError {
    fn from(e: nostr_sdk::pool::relay::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::pool::pool::Error> for NostrSdkError {
    fn from(e: nostr_sdk::pool::pool::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::database::DatabaseError> for NostrSdkError {
    fn from(e: nostr_sdk::database::DatabaseError) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_connect::error::Error> for NostrSdkError {
    fn from(e: nostr_connect::error::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_sdk::nwc::Error> for NostrSdkError {
    fn from(e: nostr_sdk::nwc::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<async_utility::thread::Error> for NostrSdkError {
    fn from(e: async_utility::thread::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}

impl From<nostr_relay_builder::error::Error> for NostrSdkError {
    fn from(e: nostr_relay_builder::error::Error) -> NostrSdkError {
        Self::Generic(e.to_string())
    }
}
