use nostr::event::Event;
use nostr::secp256k1::schnorr::Signature;
use nostr::{EventId, Kind, PublicKey, SingleLetterTag, Tags, Timestamp};
use rusqlite::Row;

use crate::error::Error;

#[derive(Debug, Clone)]
pub(crate) struct EventDb {
    pub id: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub created_at: i64,
    pub kind: i64,
    pub content: String,
    pub tags: String,
    pub sig: Vec<u8>,
}

impl EventDb {
    pub(crate) fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get("id")?,
            pubkey: row.get("pubkey")?,
            created_at: row.get("created_at")?,
            kind: row.get("kind")?,
            content: row.get("content")?,
            tags: row.get("tags")?,
            sig: row.get("sig")?,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event(self) -> Result<Event, Error> {
        let id: EventId = EventId::from_slice(&self.id)?;
        let pubkey: PublicKey = PublicKey::from_slice(&self.pubkey)?;
        let created_at: Timestamp = self.created_at.try_into()?;
        let kind: Kind = Kind::from_u16(self.kind.try_into()?);
        let tags: Tags = serde_json::from_str(&self.tags)?;
        let sig: Signature = Signature::from_slice(&self.sig)?;

        Ok(Event::new(
            id,
            pubkey,
            created_at,
            kind,
            tags,
            self.content,
            sig,
        ))
    }
}

pub(crate) struct EventTagDb<'a> {
    pub event_id: &'a [u8],
    pub tag_name: SingleLetterTag,
    pub tag_value: &'a str,
}

pub(crate) fn extract_tags(event: &Event) -> impl Iterator<Item = EventTagDb> {
    event.tags.iter().filter_map(|tag| {
        if let (Some(kind), Some(content)) = (tag.single_letter_tag(), tag.content()) {
            Some(EventTagDb {
                event_id: event.id.as_bytes(),
                tag_name: kind,
                tag_value: content,
            })
        } else {
            None
        }
    })
}
