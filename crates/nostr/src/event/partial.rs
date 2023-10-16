// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Partial Event for fast deserialization and signature verification

use core::fmt;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, Message, Secp256k1, Verification, XOnlyPublicKey};

#[cfg(feature = "std")]
use crate::SECP256K1;
use crate::{EventId, JsonUtil};

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
    /// Id
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
}

impl JsonUtil for PartialEvent {
    type Err = Error;
}
