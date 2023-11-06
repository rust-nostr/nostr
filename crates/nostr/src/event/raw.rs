// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Raw Event

use alloc::string::String;
use alloc::vec::Vec;
use core::str::FromStr;

use crate::Timestamp;

use super::kind::EPHEMERAL_RANGE;

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
