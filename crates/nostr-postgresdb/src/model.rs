use crate::schema::nostr::event_tags;
use crate::schema::nostr::events;
use diesel::prelude::*;
use nostr::event::Event;
use nostr_database::DatabaseError;
use nostr_database::FlatBufferBuilder;
use nostr_database::FlatBufferEncode;

/// DB representation of [`Event`]
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = events, check_for_backend(diesel::pg::Pg))]
pub struct EventDb {
    pub id: String,
    pub pubkey: String,
    pub created_at: i64,
    pub kind: i64,
    pub payload: Vec<u8>,
    pub signature: String,
    pub deleted: bool,
}

/// DB representation of [`EventTag`]
#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = event_tags, check_for_backend(diesel::pg::Pg))]
pub struct EventTagDb {
    pub tag: String,
    pub tag_value: String,
    pub event_id: String,
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
        let serialized = value.encode(&mut FlatBufferBuilder::new()).to_vec();
        Ok(Self {
            event: EventDb {
                id: value.id.to_string(),
                pubkey: value.pubkey.to_string(),
                created_at: value.created_at.as_u64() as i64,
                kind: value.kind.as_u16() as i64,
                payload: serialized,
                signature: value.sig.to_string(),
                deleted: false,
            },
            tags: extract_tags(value),
        })
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
                    event_id: event.id.to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}
