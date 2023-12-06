// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Raw Event

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{self, XOnlyPublicKey};

use super::id;
use super::kind::EPHEMERAL_RANGE;
use crate::nips::nip01::Coordinate;
use crate::{Event, EventId, Kind, Tag, Timestamp};

/// [`RawEvent`] error
#[derive(Debug)]
pub enum Error {
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Event ID error
    EventId(id::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::EventId(e) => write!(f, "Event ID: {e}"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<id::Error> for Error {
    fn from(e: id::Error) -> Self {
        Self::EventId(e)
    }
}

/// Raw Event
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawEvent {
    /// Id
    pub id: [u8; 32],
    /// Author
    pub pubkey: [u8; 32],
    /// Timestamp (seconds)
    pub created_at: u64,
    /// Kind
    pub kind: u64,
    /// Vector of [`Tag`]
    pub tags: Vec<Vec<String>>,
    /// Content
    pub content: String,
    /// Signature
    pub sig: [u8; 64],
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

    /// Check if event [`Kind`] is `Ephemeral`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn is_ephemeral(&self) -> bool {
        EPHEMERAL_RANGE.contains(&self.kind)
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

impl TryFrom<RawEvent> for Event {
    type Error = Error;
    fn try_from(value: RawEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            id: EventId::from_slice(&value.id)?,
            pubkey: XOnlyPublicKey::from_slice(&value.pubkey)?,
            created_at: Timestamp::from(value.created_at),
            kind: Kind::from(value.kind),
            tags: value
                .tags
                .into_iter()
                .filter_map(|tag| Tag::parse(tag).ok())
                .collect(),
            content: value.content,
            sig: Signature::from_slice(&value.sig)?,
        })
    }
}

impl From<Event> for RawEvent {
    fn from(event: Event) -> Self {
        Self {
            id: event.id.to_bytes(),
            pubkey: event.pubkey.serialize(),
            created_at: event.created_at.as_u64(),
            kind: event.kind.as_u64(),
            tags: event.tags.into_iter().map(|t| t.as_vec()).collect(),
            content: event.content,
            sig: *event.sig.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "std")]
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_event_expired() {
        let raw = RawEvent {
            id: [0u8; 32],
            pubkey: [0u8; 32],
            created_at: 0,
            kind: 1,
            tags: vec![vec!["expiration".to_string(), "12345".to_string()]],
            content: String::new(),
            sig: [0u8; 64],
        };
        let now = Timestamp::now();
        assert!(raw.is_expired(&now));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_event_not_expired() {
        let now = Timestamp::now();
        let expiry_date: u64 = now.as_u64() * 2;

        let raw = RawEvent {
            id: [0u8; 32],
            pubkey: [0u8; 32],
            created_at: 0,
            kind: 1,
            tags: vec![vec!["expiration".to_string(), expiry_date.to_string()]],
            content: String::new(),
            sig: [0u8; 64],
        };

        assert!(!raw.is_expired(&now));
    }
}
