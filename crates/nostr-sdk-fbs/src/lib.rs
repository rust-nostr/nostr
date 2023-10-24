// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Flatbuffers

use event_generated::event_fbs::{Fixed32Bytes, Fixed64Bytes};
pub use flatbuffers::FlatBufferBuilder;
use flatbuffers::InvalidFlatbuffer;
use nostr::secp256k1::schnorr::Signature;
use nostr::secp256k1::{self, XOnlyPublicKey};
use nostr::{Event, EventId, Kind, Timestamp};
use thiserror::Error;

#[allow(unused_imports, dead_code)]
mod event_generated;

pub use self::event_generated::event_fbs;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InvalidFlatbuffer(#[from] InvalidFlatbuffer),
    #[error(transparent)]
    EventId(#[from] nostr::event::id::Error),
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    #[error("not found")]
    NotFound,
}

pub trait FlatBufferUtils: Sized {
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8];
    fn decode(buf: &[u8]) -> Result<Self, Error>;
}

impl FlatBufferUtils for Event {
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8] {
        fbb.reset();

        let id = Fixed32Bytes::new(&self.id.to_bytes());
        let pubkey = Fixed32Bytes::new(&self.pubkey.serialize());
        let sig = Fixed64Bytes::new(self.sig.as_ref());
        let args = event_fbs::EventArgs {
            id: Some(&id),
            pubkey: Some(&pubkey),
            created_at: self.created_at.as_u64(),
            kind: self.kind.as_u64(),
            tags: None,    // TODO
            content: None, // TODO
            sig: Some(&sig),
        };

        let offset = event_fbs::Event::create(fbb, &args);

        event_fbs::finish_event_buffer(fbb, offset);

        fbb.finished_data()
    }

    fn decode(buf: &[u8]) -> Result<Self, Error> {
        let ev = event_fbs::root_as_event(buf)?;
        Ok(Self {
            id: EventId::from_slice(&ev.id().ok_or(Error::NotFound)?.0)?,
            pubkey: XOnlyPublicKey::from_slice(&ev.pubkey().ok_or(Error::NotFound)?.0)?,
            created_at: Timestamp::from(ev.created_at()),
            kind: Kind::from(ev.kind()),
            tags: Vec::new(),       // TODO
            content: String::new(), // TODO
            sig: Signature::from_slice(&ev.sig().ok_or(Error::NotFound)?.0)?,
        })
    }
}
