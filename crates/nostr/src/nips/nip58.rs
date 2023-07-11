//! NIP58
//!
//! <https://github.com/nostr-protocol/nips/blob/master/58.md>

use core::fmt;

use crate::{Event, Kind, Tag, UncheckedUrl};
use secp256k1::XOnlyPublicKey;

#[derive(Debug)]
/// [`BadgeAward`] error
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
    /// Badge awards lack the awarded public key
    BadgeAwardMissingATag,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "invalid length"),
            Self::InvalidKind => write!(f, "invalid kind"),
            Self::IdentifierTagNotFound => write!(f, "identifier tag not found"),
            Self::MismatchedBadgeDefinitionOrAward => write!(f, "mismatched badge definition/award"),
            Self::BadgeAwardsLackAwardedPublicKey => write!(f, "badge award events lack the awarded public keybadge award events lack the awarded public key"),
            Self::BadgeAwardMissingATag => write!(f, "badge award event lacks `a` tag"),
        }
    }
}

/// Simple struct to hold `width` x `height`.
pub struct ImageDimensions(pub u64, pub u64);

/// Helper function to filter events for a specific [`Kind`]
pub fn filter_for_kind(events: Vec<Event>, kind_needed: &Kind) -> Vec<Event> {
    events
        .into_iter()
        .filter(|e| e.kind == *kind_needed)
        .collect()
}

/// Helper function to extract an identifier tag from an array of tags
pub fn extract_identifier(tags: Vec<Tag>) -> Option<Tag> {
    tags.iter()
        .find(|tag| matches!(tag, Tag::Identifier(_)))
        .cloned()
}

/// Helper function to extract the awarded public key from an array of PubKey tags
pub fn extract_awarded_public_key(
    tags: &[Tag],
    awarded_public_key: &XOnlyPublicKey,
) -> Option<(XOnlyPublicKey, Option<UncheckedUrl>)> {
    tags.iter().find_map(|t| match t {
        Tag::PubKey(pub_key, unchecked_url) if pub_key == awarded_public_key => {
            Some((*pub_key, unchecked_url.clone()))
        }
        _ => None,
    })
}
