use std::sync::{Mutex, OnceLock};

use diesel::prelude::*;
use nostr::event::Event;
use nostr_database::{DatabaseError, FlatBufferBuilder, FlatBufferEncode};

#[cfg(feature = "mysql")]
use crate::schema::mysql::{event_tags, events};
#[cfg(feature = "postgres")]
use crate::schema::postgres::{event_tags, events};
#[cfg(feature = "sqlite")]
use crate::schema::sqlite::{event_tags, events};

/// DB representation of [`Event`]
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = events)]
pub struct EventDb {
    pub id: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub created_at: i64,
    pub kind: i64,
    pub payload: Vec<u8>,
    pub deleted: bool,
}

/// DB representation of [`EventTag`]
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = event_tags)]
pub struct EventTagDb {
    pub tag: String,
    pub tag_value: String,
    pub event_id: Vec<u8>,
}

/// A data container for extracting data from [`Event`] and its tags
#[derive(Debug, Clone)]
pub struct EventDataDb {
    pub event: EventDb,
    pub tags: Vec<EventTagDb>,
}

impl TryFrom<&Event> for EventDataDb {
    type Error = DatabaseError;
    fn try_from(value: &Event) -> Result<Self, Self::Error> {
        Ok(Self {
            event: EventDb {
                id: value.id.as_bytes().to_vec(),
                pubkey: value.pubkey.as_bytes().to_vec(),
                created_at: value.created_at.as_u64() as i64,
                kind: value.kind.as_u16() as i64,
                payload: encode_payload(value),
                deleted: false,
            },
            tags: extract_tags(value),
        })
    }
}

fn encode_payload(value: &Event) -> Vec<u8> {
    static FB_BUILDER: OnceLock<Mutex<FlatBufferBuilder>> = OnceLock::new();
    match FB_BUILDER
        .get_or_init(|| Mutex::new(FlatBufferBuilder::new()))
        .lock()
    {
        Ok(mut fb_builder) => value.encode(&mut fb_builder).to_vec(),
        Err(_) => value.encode(&mut FlatBufferBuilder::new()).to_vec(),
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
