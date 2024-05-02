// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Profile

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use std::borrow::Cow;

use crate::{Metadata, PublicKey};

/// Profile
#[derive(Debug, Clone)]
pub struct Profile {
    public_key: PublicKey,
    metadata: Metadata,
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.public_key == other.public_key
    }
}

impl Eq for Profile {}

impl PartialOrd for Profile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Profile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(&other.name())
    }
}

impl Hash for Profile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.public_key.hash(state)
    }
}

impl From<PublicKey> for Profile {
    fn from(public_key: PublicKey) -> Self {
        Self::new(public_key, Metadata::default())
    }
}

impl Profile {
    /// Compose new profile
    pub fn new(public_key: PublicKey, metadata: Metadata) -> Self {
        Self {
            public_key,
            metadata,
        }
    }

    /// Get profile public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get profile metadata
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Get profile name
    ///
    /// Steps (go to next step if field is `None` or `empty`):
    /// * Check `display_name` field
    /// * Check `name` field
    /// * Return cutted public key (ex. `00000000:00000002`)
    pub fn name(&self) -> Cow<str> {
        if let Some(display_name) = &self.metadata.display_name {
            if !display_name.is_empty() {
                return Cow::Borrowed(display_name);
            }
        }

        if let Some(name) = &self.metadata.name {
            if !name.is_empty() {
                return Cow::Borrowed(name);
            }
        }

        Cow::Owned(cut_public_key(&self.public_key))
    }
}

/// Get the first and last 8 chars of a [`PublicKey`]
///
/// Ex. `00000000:00000002`
pub fn cut_public_key(pk: &PublicKey) -> String {
    let pk: String = pk.to_hex();
    format!("{}:{}", &pk[0..8], &pk[pk.len() - 8..])
}
