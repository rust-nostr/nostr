// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP25: Reactions
//!
//! <https://github.com/nostr-protocol/nips/blob/master/25.md>

use super::nip01::Coordinate;
use crate::{Event, EventId, Kind, PublicKey, RelayUrl, Tag, TagStandard, Tags};

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
            coordinate: event.coordinate().map(|c| c.into_owned()),
            kind: Some(event.kind),
            relay_hint,
        }
    }

    pub(crate) fn into_tags(self) -> Tags {
        let mut tags: Tags = Tags::with_capacity(
            2 + usize::from(self.coordinate.is_some()) + usize::from(self.kind.is_some()),
        );

        // Serialization order: keep the `e` and `a` tags together, followed by the `p` and other tags.

        tags.push(Tag::from_standardized_without_cell(TagStandard::Event {
            event_id: self.event_id,
            relay_url: self.relay_hint.clone(),
            public_key: Some(self.public_key),
            marker: None,
            uppercase: false,
        }));

        if let Some(coordinate) = self.coordinate {
            tags.push(Tag::coordinate(coordinate, self.relay_hint.clone()));
        }

        tags.push(Tag::from_standardized_without_cell(
            TagStandard::PublicKey {
                public_key: self.public_key,
                relay_url: self.relay_hint,
                alias: None,
                uppercase: false,
            },
        ));

        if let Some(kind) = self.kind {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Kind {
                kind,
                uppercase: false,
            }));
        }

        tags
    }
}
