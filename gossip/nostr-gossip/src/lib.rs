// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Gossip

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![warn(clippy::large_futures)]

use std::any::Any;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;

use nostr::prelude::*;

pub mod error;
pub mod flags;
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
    /// No relay list is currently stored, and no fetch attempt has been tracked yet.
    Missing,
    /// The public key data is updated
    Updated,
    /// The public key data is outdated
    Outdated {
        /// The timestamp of the relay list event that is currently stored
        created_at: Option<Timestamp>,
    },
}

impl GossipPublicKeyStatus {
    /// Check if the public key is missing.
    #[inline]
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    /// Check if the public key is updated.
    #[inline]
    pub fn is_updated(&self) -> bool {
        matches!(self, Self::Updated)
    }

    /// Check if the public key is outdated.
    #[inline]
    pub fn is_outdated(&self) -> bool {
        matches!(self, Self::Outdated { .. })
    }
}

/// Allowed gossip relay types during selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GossipAllowedRelays {
    /// Allow tor onion relays (default: true)
    pub onion: bool,
    /// Allow local network relays (default: false)
    pub local: bool,
    /// Allow relays without SSL/TLS encryption (default: true)
    pub without_tls: bool,
}

impl Default for GossipAllowedRelays {
    fn default() -> Self {
        Self {
            onion: true,
            local: false,
            without_tls: true,
        }
    }
}

impl GossipAllowedRelays {
    /// Check if a relay URL is allowed.
    pub fn is_allowed(&self, relay_url: &RelayUrl) -> bool {
        if !self.onion && relay_url.is_onion() {
            return false;
        }

        if !self.local && relay_url.is_local_addr() {
            return false;
        }

        if !self.without_tls && !relay_url.scheme().is_secure() {
            return false;
        }

        true
    }
}

/// Best relay selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BestRelaySelection {
    /// Get all the best relays for **reading** and **writing** events (NIP-65)
    All {
        /// Limit for read relays
        read: u8,
        /// Limit for write relays
        write: u8,
        /// Limit for hints
        hints: u8,
        /// Limit for most received relays
        most_received: u8,
    },
    /// Get the best relays for **reading** events (NIP-65)
    Read {
        /// Limit
        limit: u8,
    },
    /// Get the best relays for **writing** events (NIP-65)
    Write {
        /// Limit
        limit: u8,
    },
    /// Get the best relays for **reading** and **writing** private messages (NIP-17)
    PrivateMessage {
        /// Limit
        limit: u8,
    },
    /// Relays found in hints
    Hints {
        /// Limit
        limit: u8,
    },
    /// Relays that received most events
    MostReceived {
        /// Limit
        limit: u8,
    },
}

/// Outdated public key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutdatedPublicKey {
    /// The public key that has been marked as outdated.
    pub public_key: PublicKey,
    /// The timestamp of the last check that has been made for this public key for a certain list kind.
    pub timestamp: Timestamp,
}

impl PartialOrd for OutdatedPublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OutdatedPublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.timestamp == other.timestamp {
            self.public_key.cmp(&other.public_key)
        } else {
            self.timestamp.cmp(&other.timestamp)
        }
    }
}

impl OutdatedPublicKey {
    /// New outdated public key
    #[inline]
    pub fn new(public_key: PublicKey, timestamp: Timestamp) -> Self {
        Self {
            public_key,
            timestamp,
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

    /// Get up to `limit` outdated public keys for the specified list kind.
    fn outdated_public_keys(
        &self,
        list: GossipListKind,
        limit: NonZeroUsize,
    ) -> BoxedFuture<Result<BTreeSet<OutdatedPublicKey>, GossipError>>;

    /// Get the best relays for a [`PublicKey`].
    fn get_best_relays<'a>(
        &'a self,
        public_key: &'a PublicKey,
        selection: BestRelaySelection,
        allowed: GossipAllowedRelays,
    ) -> BoxedFuture<'a, Result<HashSet<RelayUrl>, GossipError>>;
}

#[doc(hidden)]
pub trait IntoNostrGossip {
    fn into_nostr_gossip(self) -> Arc<dyn NostrGossip>;
}

impl IntoNostrGossip for Arc<dyn NostrGossip> {
    fn into_nostr_gossip(self) -> Arc<dyn NostrGossip> {
        self
    }
}

impl<T> IntoNostrGossip for T
where
    T: NostrGossip + Sized + 'static,
{
    fn into_nostr_gossip(self) -> Arc<dyn NostrGossip> {
        Arc::new(self)
    }
}

impl<T> IntoNostrGossip for Arc<T>
where
    T: NostrGossip + 'static,
{
    fn into_nostr_gossip(self) -> Arc<dyn NostrGossip> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_relays() {
        let clearnet = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let clearnet_no_tls = RelayUrl::parse("ws://relay.example.com").unwrap();
        let local = RelayUrl::parse("ws://192.168.1.11:7777").unwrap();
        let onion =
            RelayUrl::parse("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion")
                .unwrap();

        // All except local
        let allowed = GossipAllowedRelays {
            onion: true,
            local: false,
            without_tls: true,
        };
        assert!(allowed.is_allowed(&clearnet));
        assert!(allowed.is_allowed(&clearnet_no_tls));
        assert!(!allowed.is_allowed(&local));
        assert!(allowed.is_allowed(&onion));

        // Allow only clearnet with TLS
        let allowed = GossipAllowedRelays {
            onion: false,
            local: false,
            without_tls: false,
        };
        assert!(allowed.is_allowed(&clearnet));
        assert!(!allowed.is_allowed(&clearnet_no_tls));
        assert!(!allowed.is_allowed(&local));
        assert!(!allowed.is_allowed(&onion));
    }
}
