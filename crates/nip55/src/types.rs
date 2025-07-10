// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Types for Android signer operations

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Permission type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PermissionType {
    SignEvent,
    Nip44Encrypt,
}

impl PermissionType {
    fn parse(permission: &str) -> Result<Self, Error> {
        match permission {
            "sign_event" => Ok(Self::SignEvent),
            "nip44_encrypt" => Ok(Self::Nip44Encrypt),
            _ => Err(Error::UnknownPermission),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::SignEvent => "sign_event",
            Self::Nip44Encrypt => "nip44_encrypt",
        }
    }
}

impl Serialize for PermissionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PermissionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Permission for signer operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Type of permission
    #[serde(rename = "type")]
    pub permission_type: PermissionType,
    /// Optional kind for specific event types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<u32>,
}

impl Permission {
    /// Create a new permission for signing events
    pub fn sign_event(kind: Option<u32>) -> Self {
        Self {
            permission_type: PermissionType::SignEvent,
            kind,
        }
    }

    /// Create a new permission for NIP-44 decryption
    pub fn nip44_decrypt() -> Self {
        Self {
            permission_type: PermissionType::Nip44Encrypt,
            kind: None,
        }
    }
}
