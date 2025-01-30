// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP58: Badges
//!
//! <https://github.com/nostr-protocol/nips/blob/master/58.md>

use alloc::vec::Vec;
use core::fmt;

use crate::types::RelayUrl;
use crate::{Event, Kind, PublicKey, Tag, TagStandard};

#[derive(Debug, PartialEq, Eq)]
/// Badge Award error
pub enum Error {
    /// Invalid length
    InvalidLength,
    /// Invalid kind
    InvalidKind,
    /// Identifier tag not found
    IdentifierTagNotFound,
    /// Mismatched badge definition or award
    MismatchedBadgeDefinitionOrAward,
    /// Badge awards lack the awarded public key
    BadgeAwardsLackAwardedPublicKey,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "invalid length"),
            Self::InvalidKind => write!(f, "invalid kind"),
            Self::IdentifierTagNotFound => write!(f, "identifier tag not found"),
            Self::MismatchedBadgeDefinitionOrAward => write!(f, "mismatched badge definition/award"),
            Self::BadgeAwardsLackAwardedPublicKey => write!(f, "badge award events lack the awarded public keybadge award events lack the awarded public key"),
        }
    }
}

/// Helper function to filter events for a specific [`Kind`]
#[inline]
pub(crate) fn filter_for_kind(events: Vec<Event>, kind_needed: &Kind) -> Vec<Event> {
    events
        .into_iter()
        .filter(|e| &e.kind == kind_needed)
        .collect()
}

/// Helper function to extract the awarded public key from an array of PubKey tags
pub(crate) fn extract_awarded_public_key<'a>(
    tags: &'a [Tag],
    awarded_public_key: &PublicKey,
) -> Option<(&'a PublicKey, &'a Option<RelayUrl>)> {
    tags.iter().find_map(|t| match t.as_standardized() {
        Some(TagStandard::PublicKey {
            public_key,
            relay_url,
            ..
        }) if public_key == awarded_public_key => Some((public_key, relay_url)),
        _ => None,
    })
}
