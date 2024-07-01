// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Profile metadata

use alloc::borrow::Cow;
use alloc::string::String;
use core::cmp::Ordering;
use core::ops::Deref;

use crate::{Metadata, Profile, PublicKey};

impl PartialOrd for Profile<Metadata> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Profile<Metadata> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(&other.name())
    }
}

impl Profile<Metadata> {
    /// Get profile metadata
    #[inline]
    pub fn metadata(&self) -> &Metadata {
        self.deref()
    }

    /// Get profile name
    ///
    /// Steps (go to next step if field is `None` or `empty`):
    /// * Check `display_name` field
    /// * Check `name` field
    /// * Return cut public key (ex. `00000000:00000002`)
    pub fn name(&self) -> Cow<'_, str> {
        if let Some(display_name) = &self.data.display_name {
            if !display_name.is_empty() {
                return Cow::Borrowed(display_name);
            }
        }

        if let Some(name) = &self.data.name {
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
fn cut_public_key(pk: &PublicKey) -> String {
    let pk = pk.to_hex();
    format!("{}:{}", &pk[0..8], &pk[pk.len() - 8..])
}
