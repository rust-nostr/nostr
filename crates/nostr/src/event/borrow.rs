// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Borrowed Event

use alloc::vec::Vec;
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use secp256k1::schnorr::Signature;

use super::tag::cow::CowTag;
use crate::{Event, EventId, Kind, PublicKey, Tags, Timestamp};

/// Borrowed event
#[derive(Debug, Clone)]
pub struct EventBorrow<'a> {
    /// Event ID
    pub id: &'a [u8; 32],
    /// Author
    pub pubkey: &'a [u8; 32],
    /// UNIX timestamp (seconds)
    pub created_at: Timestamp,
    /// Kind
    pub kind: u16,
    /// Tag list
    pub tags: Vec<CowTag<'a>>,
    /// Content
    pub content: &'a str,
    /// Signature
    pub sig: &'a [u8; 64],
}

impl PartialEq for EventBorrow<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for EventBorrow<'_> {}

impl PartialOrd for EventBorrow<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EventBorrow<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.created_at != other.created_at {
            // Descending order
            // Lookup ID: EVENT_ORD_IMPL
            self.created_at.cmp(&other.created_at).reverse()
        } else {
            self.id.cmp(other.id)
        }
    }
}

impl Hash for EventBorrow<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl EventBorrow<'_> {
    /// Into owned event
    pub fn into_owned(self) -> Event {
        Event::new(
            EventId::from_byte_array(*self.id),
            PublicKey::from_byte_array(*self.pubkey),
            self.created_at,
            Kind::from_u16(self.kind),
            Tags::new(self.tags.into_iter().map(|t| t.into_owned()).collect()),
            self.content,
            // SAFETY: signature panic only if it's not 64 byte long
            Signature::from_slice(self.sig.as_slice()).expect("valid signature"),
        )
    }
}
