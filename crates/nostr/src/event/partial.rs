// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Partial Event for fast deserialization and signature verification

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Message, Secp256k1, Verification, XOnlyPublicKey};

#[cfg(feature = "std")]
use crate::SECP256K1;
use crate::{Event, EventId, JsonUtil, Kind, Tag, Timestamp};

/// [`PartialEvent`] error
#[derive(Debug)]
pub enum Error {
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
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
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
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

/// Partial event
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartialEvent {
    /// ID
    pub id: EventId,
    /// Author
    pub pubkey: XOnlyPublicKey,
    /// Signature
    pub sig: Signature,
}

impl PartialEvent {
    /// Verify [`Signature`]
    #[cfg(feature = "std")]
    pub fn verify_signature(&self) -> Result<(), Error> {
        self.verify_signature_with_ctx(&SECP256K1)
    }

    /// Verify [`Signature`]
    pub fn verify_signature_with_ctx<C>(&self, secp: &Secp256k1<C>) -> Result<(), Error>
    where
        C: Verification,
    {
        // Verify signature
        let message = Message::from_slice(self.id.as_bytes())?;
        secp.verify_schnorr(&self.sig, &message, &self.pubkey)
            .map_err(|_| Error::InvalidSignature)
    }

    /// Merge [`MissingPartialEvent`] and compose [`Event`]
    pub fn merge(&self, missing: MissingPartialEvent) -> Event {
        Event {
            id: self.id,
            pubkey: self.pubkey,
            created_at: missing.created_at,
            kind: missing.kind,
            tags: missing.tags,
            content: missing.content,
            sig: self.sig,
            #[cfg(feature = "nip03")]
            ots: missing.ots,
        }
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
    /// OpenTimestamps Attestations
    #[cfg(feature = "nip03")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ots: Option<String>,
}

impl JsonUtil for MissingPartialEvent {
    type Err = Error;
}
