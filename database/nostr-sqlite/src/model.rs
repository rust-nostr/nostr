use nostr::event::Event;
use nostr::secp256k1::schnorr::Signature;
use nostr::{EventId, Kind, PublicKey, SingleLetterTag, Tags, Timestamp};
use sqlx::types::Json;
use sqlx::FromRow;

use crate::error::Error;

#[derive(Debug, Clone, FromRow)]
pub(crate) struct EventDb {
    pub id: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub created_at: i64,
    pub kind: u16,
    pub content: String,
    pub tags: Json<Tags>,
    pub sig: Vec<u8>,
}

impl EventDb {
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_event(self) -> Result<Event, Error> {
        let id: EventId = EventId::from_slice(&self.id)?;
        let pubkey: PublicKey = PublicKey::from_slice(&self.pubkey)?;
        let created_at: Timestamp = self.created_at.try_into()?;
        let kind: Kind = Kind::from_u16(self.kind);
        let tags: Tags = self.tags.0;
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
