// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database Flatbuffers

use std::collections::HashSet;

pub use flatbuffers::FlatBufferBuilder;
use flatbuffers::InvalidFlatbuffer;
use nostr::secp256k1::schnorr::Signature;
use nostr::secp256k1::{self, XOnlyPublicKey};
use nostr::{Event, EventId, Kind, Tag, Timestamp, Url};
use thiserror::Error;

#[allow(unused_imports, dead_code, clippy::all, unsafe_code, missing_docs)]
mod event_generated;
#[allow(unused_imports, dead_code, clippy::all, unsafe_code, missing_docs)]
mod event_seen_by_generated;

use self::event_generated::event_fbs;
use self::event_seen_by_generated::event_seen_by_fbs;
use crate::raw::RawEvent;

/// FlatBuffers Error
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid FlatBuffer
    #[error(transparent)]
    InvalidFlatbuffer(#[from] InvalidFlatbuffer),
    #[error(transparent)]
    /// Event ID error
    EventId(#[from] nostr::event::id::Error),
    /// Tag error
    #[error(transparent)]
    Tag(#[from] nostr::event::tag::Error),
    /// Secp256k1 error
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    /// Not found
    #[error("not found")]
    NotFound,
}

/// FlatBuffer Encode trait
pub trait FlatBufferEncode {
    /// FlatBuffer encode
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8];
}

/// FlatBuffer Decode trait
pub trait FlatBufferDecode: Sized {
    /// FlatBuffer decode
    fn decode(buf: &[u8]) -> Result<Self, Error>;
}

impl FlatBufferEncode for Event {
    #[tracing::instrument(skip_all, level = "trace")]
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8] {
        fbb.reset();

        let id = event_fbs::Fixed32Bytes::new(&self.id().to_bytes());
        let pubkey = event_fbs::Fixed32Bytes::new(&self.author().serialize());
        let sig = event_fbs::Fixed64Bytes::new(self.signature().as_ref());
        let tags = self
            .iter_tags()
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
            created_at: self.created_at().as_u64(),
            kind: self.kind().as_u64(),
            tags: Some(fbb.create_vector(&tags)),
            content: Some(fbb.create_string(self.content())),
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

        Ok(Self::new(
            EventId::from_slice(&ev.id().ok_or(Error::NotFound)?.0)?,
            XOnlyPublicKey::from_slice(&ev.pubkey().ok_or(Error::NotFound)?.0)?,
            Timestamp::from(ev.created_at()),
            Kind::from(ev.kind()),
            tags,
            ev.content().ok_or(Error::NotFound)?.to_owned(),
            Signature::from_slice(&ev.sig().ok_or(Error::NotFound)?.0)?,
        ))
    }
}

impl FlatBufferDecode for RawEvent {
    #[tracing::instrument(skip_all, level = "trace")]
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        let ev = event_fbs::root_as_event(buf)?;
        Ok(Self {
            id: ev.id().ok_or(Error::NotFound)?.0,
            pubkey: ev.pubkey().ok_or(Error::NotFound)?.0,
            created_at: Timestamp::from(ev.created_at()),
            kind: Kind::from(ev.kind()),
            tags: ev
                .tags()
                .ok_or(Error::NotFound)?
                .into_iter()
                .filter_map(|tag| match tag.data() {
                    Some(t) => {
                        if t.len() > 1 {
                            Some(t.into_iter().map(|s| s.to_owned()).collect::<Vec<String>>())
                        } else {
                            None
                        }
                    }
                    None => None,
                })
                .collect(),
        })
    }
}

impl FlatBufferEncode for HashSet<Url> {
    #[tracing::instrument(skip_all, level = "trace")]
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8] {
        fbb.reset();

        let urls: Vec<_> = self
            .iter()
            .map(|url| fbb.create_string(url.as_ref()))
            .collect();
        let args = event_seen_by_fbs::EventSeenByArgs {
            relay_urls: Some(fbb.create_vector(&urls)),
        };

        let offset = event_seen_by_fbs::EventSeenBy::create(fbb, &args);

        event_seen_by_fbs::finish_event_seen_by_buffer(fbb, offset);

        fbb.finished_data()
    }
}

impl FlatBufferDecode for HashSet<Url> {
    #[tracing::instrument(skip_all, level = "trace")]
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        let ev = event_seen_by_fbs::root_as_event_seen_by(buf)?;
        Ok(ev
            .relay_urls()
            .ok_or(Error::NotFound)?
            .into_iter()
            .filter_map(|url| Url::parse(url).ok())
            .collect::<HashSet<Url>>())
    }
}
