// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Unsigned Event

use core::fmt;

use secp256k1::schnorr::Signature;
use secp256k1::{Message, XOnlyPublicKey};
use serde::{Deserialize, Serialize};

use crate::{Event, EventId, Keys, Kind, Tag, Timestamp};

/// [`UnsignedEvent`] error
#[derive(Debug)]
pub enum Error {
    /// Key error
    Key(crate::key::Error),
    /// Error serializing or deserializing JSON data
    Json(serde_json::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Event error
    Event(super::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::Event(e) => write!(f, "{e}"),
        }
    }
}

impl From<crate::key::Error> for Error {
    fn from(e: crate::key::Error) -> Self {
        Self::Key(e)
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

impl From<super::Error> for Error {
    fn from(e: super::Error) -> Self {
        Self::Event(e)
    }
}

/// [`UnsignedEvent`] struct
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct UnsignedEvent {
    /// Id
    pub id: EventId,
    /// Author
    pub pubkey: XOnlyPublicKey,
    /// Timestamp (seconds)
    pub created_at: Timestamp,
    /// Kind
    pub kind: Kind,
    /// Vector of [`Tag`]
    pub tags: Vec<Tag>,
    /// Content
    pub content: String,
}

impl UnsignedEvent {
    /// Sign an [`UnsignedEvent`]
    pub fn sign(self, keys: &Keys) -> Result<Event, Error> {
        let message = Message::from_slice(self.id.as_bytes())?;
        Ok(Event {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig: keys.sign_schnorr(&message)?,
            #[cfg(feature = "nip03")]
            ots: None,
        })
    }

    /// Add signature to [`UnsignedEvent`]
    pub fn add_signature(self, sig: Signature) -> Result<Event, Error> {
        let event = Event {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig,
            #[cfg(feature = "nip03")]
            ots: None,
        };
        event.verify()?;
        Ok(event)
    }

    /// Deserialize from JSON string
    pub fn from_json<S>(json: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(serde_json::from_str(&json.into())?)
    }

    /// Serialize as JSON string
    pub fn as_json(&self) -> String {
        serde_json::json!(self).to_string()
    }
}
