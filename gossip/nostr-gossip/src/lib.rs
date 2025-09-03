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

/// Gossip list kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GossipListKind {
    /// NIP-17
    Nip17,
    /// NIP-65
    Nip65,
}

impl GossipListKind {
    /// Convert to event [`Kind`].
    pub fn to_event_kind(&self) -> Kind {
        match self {
            Self::Nip17 => Kind::InboxRelays,
            Self::Nip65 => Kind::RelayList,
        }
    }
}

/// Public key status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GossipPublicKeyStatus {
    /// The public key data is updated
    Updated,
    /// The public key data is outdated
    Outdated {
        /// The timestamp of the relay list event that is currently stored
        created_at: Option<Timestamp>,
    },
}

/// Best relay selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BestRelaySelection {
    /// Get all the best relays for **reading** and **writing** events (NIP-65)
    All {
        /// Limit for read relays
        read: usize,
        /// Limit for write relays
        write: usize,
        /// Limit for hints
        hints: usize,
        /// Limit for most received relays
        most_received: usize,
    },
    /// Get the best relays for **reading** events (NIP-65)
    Read {
        /// Limit
        limit: usize,
    },
    /// Get the best relays for **writing** events (NIP-65)
    Write {
        /// Limit
        limit: usize,
    },
    /// Get the best relays for **reading** and **writing** private messages (NIP-17)
    PrivateMessage {
        /// Limit
        limit: usize,
    },
    /// Relays found in hints
    Hints {
        /// Limit
        limit: usize,
    },
    /// Relays that received most events
    MostReceived {
        /// Limit
        limit: usize,
    },
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
    fn status<'a>(
        &'a self,
        public_key: &'a PublicKey,
        list: GossipListKind,
    ) -> BoxedFuture<'a, Result<GossipPublicKeyStatus, GossipError>>;

    /// Update the last check timestamp for an [`PublicKey`].
    fn update_fetch_attempt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        list: GossipListKind,
    ) -> BoxedFuture<'a, Result<(), GossipError>>;

    /// Get the best relays for a [`PublicKey`].
    fn get_best_relays<'a>(
        &'a self,
        public_key: &'a PublicKey,
        selection: BestRelaySelection,
    ) -> BoxedFuture<'a, Result<HashSet<RelayUrl>, GossipError>>;
}
