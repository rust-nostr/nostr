// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Partial Event for fast deserialization and signature verification

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use secp256k1::schnorr::Signature;

use super::raw::{self, RawEvent};
use super::tag;
use crate::{Event, EventId, JsonUtil, Kind, PublicKey, Tag, Timestamp};

/// Partial Event error
#[derive(Debug)]
pub enum Error {
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
    /// Raw event error
    RawEvent(raw::Error),
    /// Tag parse
    Tag(tag::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Invalid signature
    InvalidSignature,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "{e}"),
            Self::RawEvent(e) => write!(f, "{e}"),
            Self::Tag(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<raw::Error> for Error {
    fn from(e: raw::Error) -> Self {
        Self::RawEvent(e)
    }
}

impl From<tag::Error> for Error {
    fn from(e: tag::Error) -> Self {
        Self::Tag(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

/// Partial event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartialEvent {
    /// ID
    pub id: EventId,
    /// Author
    pub pubkey: PublicKey,
}

impl PartialEvent {
    /// Compose from [RawEvent]
    #[inline]
    pub fn from_raw(raw: &RawEvent) -> Result<Self, Error> {
        Ok(raw.try_into()?)
    }

    /// Merge [`MissingPartialEvent`] and compose [`Event`]
    pub fn merge(self, missing: MissingPartialEvent) -> Event {
        Event::new(
            self.id,
            self.pubkey,
            missing.created_at,
            missing.kind,
            missing.tags,
            missing.content,
            missing.sig,
        )
    }
}

impl JsonUtil for PartialEvent {
    type Err = Error;
}

/// Missing Partial event fields
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MissingPartialEvent {
    /// Timestamp (seconds)
    pub created_at: Timestamp,
    /// Kind
    pub kind: Kind,
    /// Vector of [`Tag`]
    pub tags: Vec<Tag>,
    /// Content
    pub content: String,
    /// Signature
    pub sig: Signature,
}

impl MissingPartialEvent {
    /// Compose from [RawEvent]
    #[inline]
    pub fn from_raw(raw: RawEvent) -> Result<Self, Error> {
        let mut tags: Vec<Tag> = Vec::with_capacity(raw.tags.len());
        for tag in raw.tags.into_iter() {
            tags.push(Tag::parse(tag)?);
        }

        Ok(Self {
            created_at: Timestamp::from(raw.created_at),
            kind: Kind::from(raw.kind),
            tags,
            content: raw.content,
            sig: Signature::from_str(&raw.sig)?,
        })
    }
}

impl JsonUtil for MissingPartialEvent {
    type Err = Error;
}
