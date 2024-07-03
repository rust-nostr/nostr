// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::net::AddrParseError;

use nostr::message::MessageHandleError;
use uniffi::Error;

pub type Result<T, E = NostrError> = std::result::Result<T, E>;

#[derive(Debug, Error, o2o::o2o)]
#[uniffi(flat_error)]
#[from_owned(std::num::ParseIntError| repeat(), return Self::Generic(@.to_string()))]
#[from_owned(std::char::ParseCharError)]
#[from_owned(nostr::key::Error)]
#[from_owned(nostr::key::vanity::Error)]
#[from_owned(MessageHandleError)]
#[from_owned(nostr::types::metadata::Error)]
#[from_owned(nostr::event::Error)]
#[from_owned(nostr::event::builder::Error)]
#[from_owned(nostr::event::unsigned::Error)]
#[from_owned(nostr::event::tag::Error)]
#[from_owned(nostr::nips::nip01::Error)]
#[from_owned(nostr::nips::nip04::Error)]
#[from_owned(nostr::nips::nip05::Error)]
#[from_owned(nostr::nips::nip06::Error)]
#[from_owned(nostr::nips::nip11::Error)]
#[from_owned(nostr::nips::nip19::Error)]
#[from_owned(nostr::nips::nip21::Error)]
#[from_owned(nostr::nips::nip26::Error)]
#[from_owned(nostr::nips::nip44::Error)]
#[from_owned(nostr::nips::nip46::Error)]
#[from_owned(nostr::nips::nip47::Error)]
#[from_owned(nostr::nips::nip49::Error)]
#[from_owned(nostr::nips::nip53::Error)]
#[from_owned(nostr::nips::nip57::Error)]
#[from_owned(nostr::nips::nip59::Error)]
#[from_owned(nostr::nips::nip90::Error)]
#[from_owned(nostr::secp256k1::Error)]
#[from_owned(nostr::types::url::ParseError)]
#[from_owned(nostr::hashes::hex::HexToArrayError)]
#[from_owned(nostr::serde_json::Error)]
#[from_owned(nostr::event::id::Error)]
#[from_owned(AddrParseError)]
pub enum NostrError {
    Generic(String),
}

impl std::error::Error for NostrError {}

impl fmt::Display for NostrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(e) => write!(f, "{e}"),
        }
    }
}
