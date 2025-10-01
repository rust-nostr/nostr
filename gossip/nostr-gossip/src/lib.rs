// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Gossip

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]

use std::any::Any;
use std::collections::HashSet;
use std::fmt::Debug;

use nostr::prelude::*;

pub mod error;
pub mod prelude;

use self::error::GossipError;

/// Public key status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GossipPublicKeyStatus {
    /// The public key data is updated
    Updated,
    /// The public key data is outdated
    Outdated,
}

impl GossipPublicKeyStatus {
    /// Check if the public key data is outdated
    #[inline]
    pub fn is_outdated(&self) -> bool {
        matches!(self, Self::Outdated)
    }
}

/// Best relay selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BestRelaySelection {
    /// Get all the best relays for **reading** and **writing** events (NIP-65)
    All,
    /// Get the best relays for **reading** events (NIP-65)
    Read,
    /// Get the best relays for **writing** events (NIP-65)
    Write,
    /// Get the best relays for **reading** and **writing** private messages (NIP-17)
    PrivateMessage,
}

impl From<RelayMetadata> for BestRelaySelection {
    fn from(metadata: RelayMetadata) -> Self {
        match metadata {
            RelayMetadata::Read => Self::Read,
            RelayMetadata::Write => Self::Write,
        }
    }
}

/// Nostr gossip trait.
pub trait NostrGossip: Any + Debug + Send + Sync {
    /// Process an [`Event`]
    ///
    /// Optionally takes the [`RelayUrl`] from where the [`Event`] comes from.
    fn process<'a>(
        &'a self,
        event: &'a Event,
        relay_url: Option<&'a RelayUrl>,
    ) -> BoxedFuture<'a, Result<(), GossipError>>;

    /// Check the [`PublicKey`] status
    fn status(
        &self,
        public_key: &PublicKey,
    ) -> BoxedFuture<Result<GossipPublicKeyStatus, GossipError>>;

    /// Update the last check timestamp for stale [`PublicKey`].
    fn update_fetch_attempt(&self, public_key: PublicKey) -> BoxedFuture<Result<(), GossipError>>;

    /// Get the best relays for a [`PublicKey`].
    fn get_best_relays<'a>(
        &'a self,
        public_key: &'a PublicKey,
        selection: BestRelaySelection,
    ) -> BoxedFuture<'a, Result<HashSet<RelayUrl>, GossipError>>;
}
