// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Messages

use alloc::string::String;
use core::fmt;

#[cfg(feature = "rand")]
use rand::RngCore;
#[cfg(all(feature = "std", feature = "os-rng"))]
use rand::TryRngCore;
#[cfg(all(feature = "std", feature = "os-rng"))]
use rand::rngs::OsRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod client;
pub mod relay;

pub use self::client::ClientMessage;
pub use self::relay::{MachineReadablePrefix, RelayMessage};
#[cfg(feature = "rand")]
use crate::util;

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
    #[cfg(all(feature = "std", feature = "os-rng"))]
    pub fn generate() -> Self {
        Self::generate_with_rng(&mut OsRng.unwrap_err())
    }

    /// Generate new random [`SubscriptionId`]
    #[inline]
    #[cfg(feature = "rand")]
    pub fn generate_with_rng<R>(rng: &mut R) -> Self
    where
        R: RngCore,
    {
        // Cut the hash and encode to hex
        Self::new(util::random_hex_string::<R, 16>(rng))
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
        serializer.serialize_str(self.as_str())
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

#[cfg(test)]
#[cfg(all(feature = "std", feature = "os-rng"))]
mod tests {
    use super::*;

    #[test]
    fn test_generate_subscription_id() {
        let id = SubscriptionId::generate();
        assert_eq!(id.as_str().len(), 32);
        assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }
}
