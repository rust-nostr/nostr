// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Profile

use core::hash::{Hash, Hasher};
use core::ops::Deref;

pub mod metadata;

use crate::PublicKey;

/// Profile
#[derive(Debug, Clone)]
pub struct Profile<T> {
    public_key: PublicKey,
    data: T,
}

impl<T> PartialEq for Profile<T> {
    fn eq(&self, other: &Self) -> bool {
        self.public_key == other.public_key
    }
}

impl<T> Eq for Profile<T> {}

impl<T> Hash for Profile<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.public_key.hash(state)
    }
}

impl<T> Profile<T> {
    /// Construct new profile
    #[inline]
    pub fn new(public_key: PublicKey, data: T) -> Self {
        Self { public_key, data }
    }

    /// Get public key
    #[inline]
    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }
}

impl<T> Deref for Profile<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
