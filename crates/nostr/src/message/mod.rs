// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Messages

use alloc::string::{String, ToString};
use core::fmt;

use hashes::Hash;
use hashes::sha256::Hash as Sha256Hash;
use secp256k1::rand::RngCore;
#[cfg(feature = "std")]
use secp256k1::rand::rngs::OsRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod client;
pub mod relay;

pub use self::client::ClientMessage;
pub use self::relay::{MachineReadablePrefix, RelayMessage};

/// Messages error
#[derive(Debug)]
pub enum MessageHandleError {
    /// Impossible to deserialize message
    Json(serde_json::Error),
    /// Invalid message format
    InvalidMessageFormat,
}

#[cfg(feature = "std")]
impl std::error::Error for MessageHandleError {}

impl fmt::Display for MessageHandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => e.fmt(f),
            Self::InvalidMessageFormat => f.write_str("Invalid format"),
        }
    }
}

impl From<serde_json::Error> for MessageHandleError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// Subscription ID
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubscriptionId(String);

impl SubscriptionId {
    /// Create new [`SubscriptionId`]
    #[inline]
    pub fn new<S>(id: S) -> Self
    where
        S: Into<String>,
    {
        Self(id.into())
    }

    /// Generate new random [`SubscriptionId`]
    #[inline]
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        Self::generate_with_rng(&mut OsRng)
    }

    /// Generate new random [`SubscriptionId`]
    pub fn generate_with_rng<R>(rng: &mut R) -> Self
    where
        R: RngCore,
    {
        // Random bytes
        let mut bytes: [u8; 32] = [0u8; 32];
        rng.fill_bytes(&mut bytes);

        // Hash random bytes
        let hash: [u8; 32] = Sha256Hash::hash(&bytes).to_byte_array();

        // Cut the hash and encode to hex
        Self::new(hex::encode(&hash[..16]))
    }

    /// Get as `&str`
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for SubscriptionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SubscriptionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id: String = String::deserialize(deserializer)?;
        Ok(Self::new(id))
    }
}
