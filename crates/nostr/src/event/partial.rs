// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Partial Event for fast deserialization and signature verification

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Message, Secp256k1, Verification, XOnlyPublicKey};

use super::raw::{self, RawEvent};
use super::tag;
#[cfg(feature = "std")]
use crate::SECP256K1;
use crate::{Event, EventId, JsonUtil, Kind, PublicKey, Tag, Timestamp};

/// [`PartialEvent`] error
#[derive(Debug)]
pub enum Error {
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Raw event error
    RawEvent(raw::Error),
    /// Tag parse
    Tag(tag::Error),
    /// Invalid signature
    InvalidSignature,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::RawEvent(e) => write!(f, "Raw event: {e}"),
            Self::Tag(e) => write!(f, "Tag: {e}"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
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

/// Partial event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartialEvent {
    /// ID
    pub id: EventId,
    /// Author
    pub pubkey: PublicKey,
    /// Signature
    pub sig: Signature,
}

impl PartialEvent {
    /// Construct from [RawEvent]
    #[inline]
    pub fn from_raw(raw: &RawEvent) -> Result<Self, Error> {
        Ok(raw.try_into()?)
    }

    /// Verify [`Signature`]
    #[inline]
    #[cfg(feature = "std")]
    pub fn verify_signature(&self) -> Result<(), Error> {
        self.verify_signature_with_ctx(&SECP256K1)
    }

    /// Verify [`Signature`]
    #[inline]
    pub fn verify_signature_with_ctx<C>(&self, secp: &Secp256k1<C>) -> Result<(), Error>
    where
        C: Verification,
    {
        // Verify signature
        let message: Message = Message::from_digest_slice(self.id.as_bytes())?;
        let public_key: &XOnlyPublicKey = self.pubkey.get_xonly_public_key()?;
        secp.verify_schnorr(&self.sig, &message, public_key)
            .map_err(|_| Error::InvalidSignature)
    }

    /// Merge [`MissingPartialEvent`] and compose [`Event`]
    pub fn merge(self, missing: MissingPartialEvent) -> Result<Event, Error> {
        let mut tags: Vec<Tag> = Vec::with_capacity(missing.tags.len());
        for tag in missing.tags.into_iter() {
            tags.push(Tag::parse(&tag)?);
        }

        Ok(Event::new(
            self.id,
            self.pubkey,
            missing.created_at,
            missing.kind,
            tags,
            missing.content,
            self.sig,
        ))
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
    pub tags: Vec<Vec<String>>,
    /// Content
    pub content: String,
}

impl MissingPartialEvent {
    /// Compose from [RawEvent]
    #[inline]
    pub fn from_raw(raw: RawEvent) -> Self {
        Self {
            created_at: Timestamp::from(raw.created_at),
            kind: Kind::from(raw.kind),
            tags: raw.tags,
            content: raw.content,
        }
    }

    /// Extract identifier (`d` tag), if exists.
    pub fn identifier(&self) -> Option<&str> {
        for tag in self.tags.iter() {
            if let Some("d") = tag.first().map(|x| x.as_str()) {
                return tag.get(1).map(|x| x.as_str());
            }
        }
        None
    }
}

impl JsonUtil for MissingPartialEvent {
    type Err = Error;
}
