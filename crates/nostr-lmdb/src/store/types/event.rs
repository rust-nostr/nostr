// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::cmp::Ordering;
use std::str::FromStr;

use nostr::prelude::*;
use nostr_database::flatbuffers::event_fbs::{Fixed32Bytes, Fixed64Bytes, StringVector};
use nostr_database::flatbuffers::{
    self, event_fbs, FlatBufferDecodeBorrowed, ForwardsUOffset, Vector,
};

use crate::store::Error;

pub struct DatabaseTag<'a> {
    buf: StringVector<'a>,
}

impl<'a> DatabaseTag<'a> {
    /// Extract tag name and value
    #[inline]
    pub fn extract(&self) -> Option<(SingleLetterTag, &str)> {
        self.buf.data().and_then(|t| {
            if t.len() >= 2 {
                let tag_name: SingleLetterTag = SingleLetterTag::from_str(t.get(0)).ok()?;
                let tag_value: &str = t.get(1);
                Some((tag_name, tag_value))
            } else {
                None
            }
        })
    }
}

pub struct DatabaseEvent<'a> {
    pub id: &'a Fixed32Bytes,
    pub pubkey: &'a Fixed32Bytes,
    pub created_at: Timestamp,
    pub kind: u16,
    pub tags: Vector<'a, ForwardsUOffset<StringVector<'a>>>,
    pub content: &'a str,
    sig: &'a Fixed64Bytes,
}

impl<'a> PartialEq for DatabaseEvent<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a> Eq for DatabaseEvent<'a> {}

impl<'a> PartialOrd for DatabaseEvent<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for DatabaseEvent<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.created_at != other.created_at {
            // Descending order
            // NOT EDIT, will break many things!!
            self.created_at.cmp(&other.created_at).reverse()
        } else {
            self.id.cmp(other.id)
        }
    }
}

impl<'a> DatabaseEvent<'a> {
    #[inline]
    pub fn id(&self) -> &[u8; 32] {
        &self.id.0
    }

    #[inline]
    pub fn author(&self) -> &[u8; 32] {
        &self.pubkey.0
    }

    #[inline]
    pub fn iter_tags(&self) -> impl Iterator<Item = DatabaseTag<'a>> {
        self.tags.iter().map(|buf| DatabaseTag { buf })
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_event(self) -> Result<Event, Error> {
        let id = EventId::from_byte_array(self.id.0);
        let pubkey = PublicKey::from_slice(&self.pubkey.0)?;
        let kind = Kind::from(self.kind);
        let tags: Vec<Tag> = self
            .tags
            .into_iter()
            .filter_map(|tag| {
                tag.data()
                    .map(|tag| Tag::parse(&tag.into_iter().collect::<Vec<&str>>()).ok())
            })
            .flatten()
            .collect();
        let sig = Signature::from_slice(&self.sig.0)?;
        Ok(Event::new(
            id,
            pubkey,
            self.created_at,
            kind,
            tags,
            self.content,
            sig,
        ))
    }
}

impl<'a> FlatBufferDecodeBorrowed<'a> for DatabaseEvent<'a> {
    fn decode(buf: &'a [u8]) -> Result<Self, flatbuffers::Error> {
        let ev = event_fbs::root_as_event(buf)?;
        Ok(Self {
            id: ev.id().ok_or(flatbuffers::Error::NotFound)?,
            pubkey: ev.pubkey().ok_or(flatbuffers::Error::NotFound)?,
            created_at: Timestamp::from_secs(ev.created_at()),
            kind: ev.kind() as u16,
            tags: ev.tags().ok_or(flatbuffers::Error::NotFound)?,
            content: ev.content().ok_or(flatbuffers::Error::NotFound)?,
            sig: ev.sig().ok_or(flatbuffers::Error::NotFound)?,
        })
    }
}
