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

/// Best relay selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BestRelaySelection {
    /// Get all the best relays
    All,
    /// Get the best relays for reading events
    Read,
    /// Get the best relays for writing events
    Write,
    /// Get the best relays for sending private messages
    PrivateMessage,
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

    /// Get the best relays for a [`PublicKey`].
    fn get_best_relays<'a>(
        &'a self,
        public_key: &'a PublicKey,
        selection: BestRelaySelection,
    ) -> BoxedFuture<'a, Result<HashSet<RelayUrl>, GossipError>>;
}
