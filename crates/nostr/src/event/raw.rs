// Copyright (c) 2021 Paul Miller
// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Raw event

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use secp256k1::schnorr::Signature;

use super::tag;
use crate::{event, key, Event, EventId, JsonUtil, Kind, PartialEvent, PublicKey, Tag, Timestamp};

/// Raw event error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Event error
    Event(event::Error),
    /// Keys error
    Keys(key::Error),
    /// Tag error
    Tag(tag::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
            Self::Keys(e) => write!(f, "{e}"),
            Self::Tag(e) => write!(f, "{e}"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Keys(e)
    }
}

impl From<tag::Error> for Error {
    fn from(e: tag::Error) -> Self {
        Self::Tag(e)
    }
}

/// Raw event
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RawEvent {
    /// ID
    pub id: String,
    /// Author
    pub pubkey: String,
    /// Timestamp (seconds)
    pub created_at: u64,
    /// Kind
    pub kind: u16,
    /// Vector of strings
    pub tags: Vec<Vec<String>>,
    /// Content
    pub content: String,
    /// Signature
    pub sig: String,
}

impl JsonUtil for RawEvent {
    type Err = serde_json::Error;
}

impl TryFrom<RawEvent> for Event {
    type Error = Error;

    fn try_from(raw: RawEvent) -> Result<Self, Self::Error> {
        let id: EventId = EventId::from_hex(&raw.id)?;
        let public_key: PublicKey = PublicKey::from_hex(&raw.pubkey)?;
        let created_at: Timestamp = Timestamp::from(raw.created_at);
        let kind: Kind = Kind::from(raw.kind);
        let tags: Vec<Tag> = raw
            .tags
            .into_iter()
            .map(Tag::parse)
            .collect::<Result<Vec<_>, _>>()?;
        let sig: Signature = Signature::from_str(&raw.sig)?;
        Ok(Self::new(
            id,
            public_key,
            created_at,
            kind,
            tags,
            raw.content,
            sig,
        ))
    }
}

impl TryFrom<&RawEvent> for PartialEvent {
    type Error = Error;

    fn try_from(raw: &RawEvent) -> Result<Self, Self::Error> {
        let id: EventId = EventId::from_hex(&raw.id)?;
        let public_key: PublicKey = PublicKey::from_hex(&raw.pubkey)?;
        Ok(Self {
            id,
            pubkey: public_key,
        })
    }
}
