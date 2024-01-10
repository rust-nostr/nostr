// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Raw Event

use core::cmp::Ordering;
use core::str::FromStr;

use nostr::nips::nip01::Coordinate;
use nostr::secp256k1::XOnlyPublicKey;
use nostr::{Event, EventId, Kind, Timestamp};

/// Raw Event
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawEvent {
    /// Id
    pub id: [u8; 32],
    /// Author
    pub pubkey: [u8; 32],
    /// Timestamp (seconds)
    pub created_at: Timestamp,
    /// Kind
    pub kind: Kind,
    /// Vector of [`Tag`]
    pub tags: Vec<Vec<String>>,
    /// Content
    pub content: String,
}

impl PartialOrd for RawEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RawEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.created_at != other.created_at {
            self.created_at.cmp(&other.created_at)
        } else {
            self.id.cmp(&other.id)
        }
    }
}

impl RawEvent {
    /// Returns `true` if the event has an expiration tag that is expired.
    /// If an event has no `Expiration` tag, then it will return `false`.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    pub fn is_expired(&self, now: &Timestamp) -> bool {
        for tag in self.tags.iter() {
            if tag.len() == 2 && tag[0] == "expiration" {
                if let Ok(timestamp) = Timestamp::from_str(&tag[1]) {
                    return &timestamp < now;
                }
                break;
            }
        }
        false
    }

    /// Extract identifier (`d` tag), if exists.
    pub fn identifier(&self) -> Option<&str> {
        for tag in self.tags.iter() {
            if let Some("d") = tag.first().map(|x| x.as_str()) {
                return tag.get(1).map(|x| x.as_str());
            }
        }
        None
    }

    /// Extract public keys from tags (`p` tag)
    pub fn public_keys(&self) -> impl Iterator<Item = XOnlyPublicKey> + '_ {
        self.tags.iter().filter_map(|tag| {
            if let Some("p") = tag.first().map(|x| x.as_str()) {
                let pk = tag.get(1)?;
                Some(XOnlyPublicKey::from_str(pk).ok()?)
            } else {
                None
            }
        })
    }

    /// Extract event IDs from tags (`e` tag)
    pub fn event_ids(&self) -> impl Iterator<Item = EventId> + '_ {
        self.tags.iter().filter_map(|tag| {
            if let Some("e") = tag.first().map(|x| x.as_str()) {
                let pk = tag.get(1)?;
                Some(EventId::from_hex(pk).ok()?)
            } else {
                None
            }
        })
    }

    /// Extract coordinates from tags (`a` tag)
    pub fn coordinates(&self) -> impl Iterator<Item = Coordinate> + '_ {
        self.tags.iter().filter_map(|tag| {
            if let Some("a") = tag.first().map(|x| x.as_str()) {
                let c = tag.get(1)?;
                Some(Coordinate::from_str(c).ok()?)
            } else {
                None
            }
        })
    }
}

impl From<&Event> for RawEvent {
    fn from(event: &Event) -> Self {
        Self {
            id: event.id().to_bytes(),
            pubkey: event.author_ref().serialize(),
            created_at: event.created_at(),
            kind: event.kind(),
            tags: event.iter_tags().map(|t| t.as_vec()).collect(),
            content: event.content().to_string(),
        }
    }
}

impl From<Event> for RawEvent {
    fn from(event: Event) -> Self {
        Self {
            id: event.id().to_bytes(),
            pubkey: event.author_ref().serialize(),
            created_at: event.created_at(),
            kind: event.kind(),
            content: event.content().to_string(),
            tags: event.into_iter_tags().map(|t| t.to_vec()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_expired() {
        let raw = RawEvent {
            id: [0u8; 32],
            pubkey: [0u8; 32],
            created_at: Timestamp::from(0),
            kind: Kind::TextNote,
            tags: vec![vec!["expiration".to_string(), "12345".to_string()]],
            content: String::new(),
        };
        let now = Timestamp::now();
        assert!(raw.is_expired(&now));
    }

    #[test]
    fn test_event_not_expired() {
        let now = Timestamp::now();
        let expiry_date: u64 = now.as_u64() * 2;

        let raw = RawEvent {
            id: [0u8; 32],
            pubkey: [0u8; 32],
            created_at: Timestamp::from(0),
            kind: Kind::TextNote,
            tags: vec![vec!["expiration".to_string(), expiry_date.to_string()]],
            content: String::new(),
        };

        assert!(!raw.is_expired(&now));
    }
}
