// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP25: Reactions
//!
//! <https://github.com/nostr-protocol/nips/blob/master/25.md>

use super::nip01::Coordinate;
use super::nip22::Nip22Tag;
use crate::event::tag::{Tag, TagCodec, TagStandard, Tags};
use crate::{Event, EventId, Kind, PublicKey, RelayUrl};

/// Reaction target
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReactionTarget {
    /// Event ID
    pub event_id: EventId,
    /// Public Key
    pub public_key: PublicKey,
    /// Coordinate
    pub coordinate: Option<Coordinate>,
    /// Kind
    pub kind: Option<Kind>,
    /// Relay hint
    pub relay_hint: Option<RelayUrl>,
}

impl ReactionTarget {
    /// Construct a new reaction target
    pub fn new(event: &Event, relay_hint: Option<RelayUrl>) -> Self {
        Self {
            event_id: event.id,
            public_key: event.pubkey,
            coordinate: event.coordinate(),
            kind: Some(event.kind),
            relay_hint,
        }
    }

    pub(crate) fn into_tags(self) -> Tags {
        let mut tags: Tags = Tags::with_capacity(
            2 + usize::from(self.coordinate.is_some()) + usize::from(self.kind.is_some()),
        );

        // Serialization order: keep the `e` and `a` tags together, followed by the `p` and other tags.

        // TODO: replace with a dedicated NIP-25 tag
        tags.push(
            Nip22Tag::Event {
                id: self.event_id,
                relay_hint: self.relay_hint.clone(),
                public_key: Some(self.public_key),
                uppercase: false,
            }
            .to_tag(),
        );

        if let Some(coordinate) = self.coordinate {
            tags.push(Tag::coordinate(coordinate, self.relay_hint.clone()));
        }

        tags.push(Tag::from_standardized(TagStandard::PublicKey {
            public_key: self.public_key,
            relay_url: self.relay_hint,
            uppercase: false,
        }));

        if let Some(kind) = self.kind {
            tags.push(Tag::from_standardized(TagStandard::Kind {
                kind,
                uppercase: false,
            }));
        }

        tags
    }
}

impl From<&Event> for ReactionTarget {
    fn from(event: &Event) -> Self {
        Self {
            event_id: event.id,
            public_key: event.pubkey,
            coordinate: event.coordinate(),
            kind: Some(event.kind),
            relay_hint: None,
        }
    }
}
