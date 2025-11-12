// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::event::Event;
use nostr_database::{FlatBufferBuilder, FlatBufferEncode};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub(crate) struct EventDb {
    pub id: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub created_at: i64,
    pub kind: i64,
    pub payload: Vec<u8>,
    pub deleted: bool,
}

impl EventDb {
    #[inline]
    pub(super) fn is_deleted(&self) -> bool {
        self.deleted
    }
}

#[derive(Debug, Clone, FromRow)]
pub(crate) struct EventTagDb {
    pub tag: String,
    pub tag_value: String,
    pub event_id: Vec<u8>,
}

/// A data container for extracting data from [`Event`] and its tags
#[derive(Debug, Clone)]
pub(crate) struct EventDataDb {
    pub event: EventDb,
    pub tags: Vec<EventTagDb>,
}

impl EventDataDb {
    pub(crate) fn from_event(event: &Event, fbb: &mut FlatBufferBuilder) -> Self {
        Self {
            event: EventDb {
                id: event.id.as_bytes().to_vec(),
                pubkey: event.pubkey.as_bytes().to_vec(),
                created_at: event.created_at.as_secs() as i64,
                kind: event.kind.as_u16() as i64,
                payload: event.encode(fbb).to_vec(),
                deleted: false,
            },
            tags: extract_tags(event),
        }
    }
}

fn extract_tags(event: &Event) -> Vec<EventTagDb> {
    event
        .tags
        .iter()
        .filter_map(|tag| {
            if let (kind, Some(content)) = (tag.kind(), tag.content()) {
                Some(EventTagDb {
                    tag: kind.to_string(),
                    tag_value: content.to_string(),
                    event_id: event.id.as_bytes().to_vec(),
                })
            } else {
                None
            }
        })
        .collect()
}
