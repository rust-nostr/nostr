// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Temp Event

#![allow(missing_docs)]

use core::cmp::Ordering;
use core::str::FromStr;

use flatbuffers::{ForwardsUOffset, Vector};
use nostr::nips::nip01::Coordinate;
use nostr::{EventId, Kind, Timestamp};

use crate::flatbuffers::StringVector;
use crate::tag_indexes::{hash, TagIndexes, TAG_INDEX_VALUE_SIZE};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TempEvent {
    pub id: [u8; 32],
    pub pubkey: [u8; 32],
    pub created_at: Timestamp,
    pub kind: Kind,
    pub tags: TagIndexes,
    pub expiration: Option<Timestamp>,
    pub identifier: Option<[u8; TAG_INDEX_VALUE_SIZE]>,
    pub event_ids: Vec<EventId>,
    pub coordinates: Vec<Coordinate>,
}

impl PartialOrd for TempEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TempEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.created_at != other.created_at {
            self.created_at.cmp(&other.created_at)
        } else {
            self.id.cmp(&other.id)
        }
    }
}

impl TempEvent {
    pub(crate) fn new<'a>(
        id: [u8; 32],
        pubkey: [u8; 32],
        created_at: u64,
        kind: u16,
        tags: Vector<'a, ForwardsUOffset<StringVector<'a>>>,
    ) -> Self {
        Self {
            id,
            pubkey,
            created_at: Timestamp::from(created_at),
            kind: Kind::from(kind),
            expiration: extract_expiration(&tags),
            identifier: extract_identifier(&tags),
            event_ids: extract_event_ids(&tags),
            coordinates: extract_coordinates(&tags),
            tags: TagIndexes::from_flatb(tags),
        }
    }

    pub(crate) fn is_expired(&self, now: &Timestamp) -> bool {
        if let Some(timestamp) = self.expiration {
            return &timestamp < now;
        }
        false
    }
}

fn extract_expiration<'a>(
    tags: &Vector<'a, ForwardsUOffset<StringVector<'a>>>,
) -> Option<Timestamp> {
    let tag = tags.iter().next()?;
    tag.data().and_then(|tag| {
        if tag.len() == 2 && tag.get(0) == "expiration" {
            Timestamp::from_str(tag.get(1)).ok()
        } else {
            None
        }
    })
}

fn extract_identifier<'a>(
    tags: &Vector<'a, ForwardsUOffset<StringVector<'a>>>,
) -> Option<[u8; TAG_INDEX_VALUE_SIZE]> {
    let tag = tags.iter().next()?;
    tag.data().and_then(|tag| {
        if tag.len() >= 2 && tag.get(0) == "d" {
            return Some(hash(tag.get(1)));
        }
        None
    })
}

fn extract_event_ids<'a>(tags: &Vector<'a, ForwardsUOffset<StringVector<'a>>>) -> Vec<EventId> {
    tags.iter()
        .filter_map(|tag| {
            tag.data().and_then(|tag| {
                if tag.len() >= 2 && tag.get(0) == "e" {
                    let pk = tag.get(1);
                    Some(EventId::from_hex(pk).ok()?)
                } else {
                    None
                }
            })
        })
        .collect()
}

fn extract_coordinates<'a>(
    tags: &Vector<'a, ForwardsUOffset<StringVector<'a>>>,
) -> Vec<Coordinate> {
    tags.iter()
        .filter_map(|tag| {
            tag.data().and_then(|tag| {
                if tag.len() >= 2 && tag.get(0) == "a" {
                    let c = tag.get(1);
                    Some(Coordinate::from_str(c).ok()?)
                } else {
                    None
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_expired() {
        let raw = TempEvent {
            id: [0u8; 32],
            pubkey: [0u8; 32],
            created_at: Timestamp::from(0),
            kind: Kind::TextNote,
            tags: TagIndexes::default(),
            expiration: Some(Timestamp::from(12345)),
            identifier: None,
            event_ids: Vec::new(),
            coordinates: Vec::new(),
        };
        let now = Timestamp::now();
        assert!(raw.is_expired(&now));
    }

    #[test]
    fn test_event_not_expired() {
        let now = Timestamp::now();
        let expiry_date: u64 = now.as_u64() * 2;

        let raw = TempEvent {
            id: [0u8; 32],
            pubkey: [0u8; 32],
            created_at: Timestamp::from(0),
            kind: Kind::TextNote,
            tags: TagIndexes::default(),
            expiration: Some(Timestamp::from(expiry_date)),
            identifier: None,
            event_ids: Vec::new(),
            coordinates: Vec::new(),
        };

        assert!(!raw.is_expired(&now));
    }
}
