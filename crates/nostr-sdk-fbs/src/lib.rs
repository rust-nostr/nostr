// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Flatbuffers

pub use flatbuffers::FlatBufferBuilder;
use flatbuffers::InvalidFlatbuffer;
use nostr::secp256k1::schnorr::Signature;
use nostr::secp256k1::{self, XOnlyPublicKey};
use nostr::{Event, EventId, Kind, Tag, Timestamp};
use thiserror::Error;

#[allow(unused_imports, dead_code, clippy::all)]
mod event_generated;

pub use self::event_generated::event_fbs;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InvalidFlatbuffer(#[from] InvalidFlatbuffer),
    #[error(transparent)]
    EventId(#[from] nostr::event::id::Error),
    #[error(transparent)]
    Tag(#[from] nostr::event::tag::Error),
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    #[error("not found")]
    NotFound,
}

pub trait FlatBufferEncode {
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8];
}

pub trait FlatBufferDecode: Sized {
    fn decode(buf: &[u8]) -> Result<Self, Error>;
}

impl FlatBufferEncode for Event {
    #[tracing::instrument(skip_all, level = "trace")]
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8] {
        fbb.reset();

        let id = event_fbs::Fixed32Bytes::new(&self.id.to_bytes());
        let pubkey = event_fbs::Fixed32Bytes::new(&self.pubkey.serialize());
        let sig = event_fbs::Fixed64Bytes::new(self.sig.as_ref());
        let tags = self
            .tags
            .iter()
            .map(|t| {
                let tags = t
                    .as_vec()
                    .iter()
                    .map(|t| fbb.create_string(t))
                    .collect::<Vec<_>>();
                let args = event_fbs::StringVectorArgs {
                    data: Some(fbb.create_vector(&tags)),
                };
                event_fbs::StringVector::create(fbb, &args)
            })
            .collect::<Vec<_>>();
        let args = event_fbs::EventArgs {
            id: Some(&id),
            pubkey: Some(&pubkey),
            created_at: self.created_at.as_u64(),
            kind: self.kind.as_u64(),
            tags: Some(fbb.create_vector(&tags)),
            content: Some(fbb.create_string(&self.content)),
            sig: Some(&sig),
        };

        let offset = event_fbs::Event::create(fbb, &args);

        event_fbs::finish_event_buffer(fbb, offset);

        fbb.finished_data()
    }
}

impl FlatBufferDecode for Event {
    #[tracing::instrument(skip_all, level = "trace")]
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        let ev = event_fbs::root_as_event(buf)?;
        let tags = ev
            .tags()
            .ok_or(Error::NotFound)?
            .into_iter()
            .filter_map(|tag| {
                tag.data()
                    .map(|tag| Tag::parse(tag.into_iter().collect::<Vec<&str>>()))
            })
            .collect::<Result<Vec<Tag>, _>>()?;

        Ok(Self {
            id: EventId::from_slice(&ev.id().ok_or(Error::NotFound)?.0)?,
            pubkey: XOnlyPublicKey::from_slice(&ev.pubkey().ok_or(Error::NotFound)?.0)?,
            created_at: Timestamp::from(ev.created_at()),
            kind: Kind::from(ev.kind()),
            tags,
            content: ev.content().ok_or(Error::NotFound)?.to_owned(),
            sig: Signature::from_slice(&ev.sig().ok_or(Error::NotFound)?.0)?,
        })
    }
}
