// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database Flatbuffers

use std::collections::HashSet;
use std::fmt;

use flatbuffers::InvalidFlatbuffer;
pub use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector};
use nostr::prelude::*;
use nostr::secp256k1;
use nostr::secp256k1::schnorr::Signature;

#[allow(unused_imports, dead_code, clippy::all, unsafe_code, missing_docs)]
mod event_generated;
#[allow(unused_imports, dead_code, clippy::all, unsafe_code, missing_docs)]
mod event_seen_by_generated;

pub use self::event_generated::event_fbs;
use self::event_seen_by_generated::event_seen_by_fbs;

/// FlatBuffers Error
#[derive(Debug)]
pub enum Error {
    /// FlatBuffer
    FlatBuffer(InvalidFlatbuffer),
    /// Tag error
    Tag(tag::Error),
    /// Key error
    Key(key::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Not found
    NotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FlatBuffer(e) => write!(f, "{e}"),
            Self::Tag(e) => write!(f, "{e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::NotFound => write!(f, "not found"),
        }
    }
}

impl From<InvalidFlatbuffer> for Error {
    fn from(e: InvalidFlatbuffer) -> Self {
        Self::FlatBuffer(e)
    }
}

impl From<tag::Error> for Error {
    fn from(e: tag::Error) -> Self {
        Self::Tag(e)
    }
}

impl From<key::Error> for Error {
    fn from(e: key::Error) -> Self {
        Self::Key(e)
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
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

/// FlatBuffer Decode trait
pub trait FlatBufferDecodeBorrowed<'a>: Sized {
    /// FlatBuffer decode
    fn decode(buf: &'a [u8]) -> Result<Self, Error>;
}

impl FlatBufferEncode for Event {
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8] {
        fbb.reset();

        let id = event_fbs::Fixed32Bytes::new(&self.id.to_bytes());
        let pubkey = event_fbs::Fixed32Bytes::new(&self.pubkey.to_bytes());
        let sig = event_fbs::Fixed64Bytes::new(self.sig.as_ref());
        let tags = self
            .tags
            .iter()
            .map(|t| {
                let tags = t
                    .as_slice()
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
            kind: self.kind.as_u16() as u64,
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
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        let ev = event_fbs::root_as_event(buf)?;
        let tags = ev
            .tags()
            .ok_or(Error::NotFound)?
            .into_iter()
            .filter_map(|tag| tag.data().map(Tag::parse))
            .collect::<Result<Vec<Tag>, _>>()?;

        Ok(Self::new(
            EventId::from_byte_array(ev.id().ok_or(Error::NotFound)?.0),
            PublicKey::from_slice(&ev.pubkey().ok_or(Error::NotFound)?.0)?,
            Timestamp::from(ev.created_at()),
            Kind::from(ev.kind() as u16),
            tags,
            ev.content().ok_or(Error::NotFound)?.to_owned(),
            Signature::from_slice(&ev.sig().ok_or(Error::NotFound)?.0)?,
        ))
    }
}

impl FlatBufferEncode for HashSet<RelayUrl> {
    fn encode<'a>(&self, fbb: &'a mut FlatBufferBuilder) -> &'a [u8] {
        fbb.reset();

        let urls: Vec<_> = self
            .iter()
            .map(|url| fbb.create_string(url.as_str()))
            .collect();
        let args = event_seen_by_fbs::EventSeenByArgs {
            relay_urls: Some(fbb.create_vector(&urls)),
        };

        let offset = event_seen_by_fbs::EventSeenBy::create(fbb, &args);

        event_seen_by_fbs::finish_event_seen_by_buffer(fbb, offset);

        fbb.finished_data()
    }
}

impl FlatBufferDecode for HashSet<RelayUrl> {
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        let ev = event_seen_by_fbs::root_as_event_seen_by(buf)?;
        Ok(ev
            .relay_urls()
            .ok_or(Error::NotFound)?
            .into_iter()
            .filter_map(|url| RelayUrl::parse(url).ok())
            .collect::<HashSet<RelayUrl>>())
    }
}
