// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Unsigned Event

use secp256k1::schnorr::Signature;
use secp256k1::{Message, XOnlyPublicKey};
use serde::{Deserialize, Serialize};

use crate::{Event, EventId, Keys, Kind, Tag, Timestamp};

/// [`UnsignedEvent`] error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Key error
    #[error(transparent)]
    Key(#[from] crate::key::Error),
    /// Error serializing or deserializing JSON data
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Secp256k1 error
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    /// Event error
    #[error(transparent)]
    Event(#[from] super::Error),
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
