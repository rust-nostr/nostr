// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Nostr Database Flatbuffers

use std::borrow::Cow;
use std::fmt;

use flatbuffers::InvalidFlatbuffer;
pub use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector};
use nostr::prelude::*;
use nostr::secp256k1;
use nostr::secp256k1::schnorr::Signature;

#[allow(unused_imports, dead_code, clippy::all, unsafe_code, missing_docs)]
mod event_generated;

pub use self::event_generated::event_fbs;

/// Missing field
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MissingField {
    /// ID
    Id,
    /// Public key
    Pubkey,
    /// Tags
    Tags,
    /// Content
    Content,
    /// Signature
    Sig,
}

impl fmt::Display for MissingField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id => write!(f, "id"),
            Self::Pubkey => write!(f, "pubkey"),
            Self::Tags => write!(f, "tags"),
            Self::Content => write!(f, "content"),
            Self::Sig => write!(f, "sig"),
        }
    }
}

/// FlatBuffers Error
#[derive(Debug)]
pub enum Error {
    /// FlatBuffer
    FlatBuffer(InvalidFlatbuffer),
    /// Tag error
    Tag(tag::Error),
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Field not found
    FieldNotFound(MissingField),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FlatBuffer(e) => write!(f, "{e}"),
            Self::Tag(e) => write!(f, "{e}"),
            Self::Secp256k1(e) => write!(f, "{e}"),
            Self::FieldNotFound(field) => write!(f, "'{field}' field not found"),
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

        let id = event_fbs::Fixed32Bytes::new(self.id.as_bytes());
        let pubkey = event_fbs::Fixed32Bytes::new(self.pubkey.as_bytes());
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
            created_at: self.created_at.as_secs(),
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
            .ok_or(Error::FieldNotFound(MissingField::Tags))?
            .into_iter()
            .filter_map(|tag| tag.data().map(Tag::parse))
            .collect::<Result<Vec<Tag>, _>>()?;

        Ok(Self::new(
            EventId::from_byte_array(ev.id().ok_or(Error::FieldNotFound(MissingField::Id))?.0),
            PublicKey::from_byte_array(
                ev.pubkey()
                    .ok_or(Error::FieldNotFound(MissingField::Pubkey))?
                    .0,
            ),
            Timestamp::from(ev.created_at()),
            Kind::from(ev.kind() as u16),
            tags,
            ev.content()
                .ok_or(Error::FieldNotFound(MissingField::Content))?
                .to_owned(),
            Signature::from_slice(&ev.sig().ok_or(Error::FieldNotFound(MissingField::Sig))?.0)?,
        ))
    }
}

impl<'a> FlatBufferDecodeBorrowed<'a> for EventBorrow<'a> {
    fn decode(buf: &'a [u8]) -> Result<Self, Error> {
        let ev = event_fbs::root_as_event(buf)?;

        let fb_tags = ev.tags().ok_or(Error::FieldNotFound(MissingField::Tags))?;
        let mut tags = Vec::with_capacity(fb_tags.len());

        for tag in fb_tags.iter().filter_map(|t| t.data()) {
            tags.push(CowTag::parse(tag.into_iter().map(Cow::Borrowed).collect())?);
        }

        Ok(Self {
            id: &ev.id().ok_or(Error::FieldNotFound(MissingField::Id))?.0,
            pubkey: &ev
                .pubkey()
                .ok_or(Error::FieldNotFound(MissingField::Pubkey))?
                .0,
            created_at: Timestamp::from_secs(ev.created_at()),
            kind: ev.kind() as u16, // TODO: should use try_into
            tags,
            content: ev
                .content()
                .ok_or(Error::FieldNotFound(MissingField::Content))?,
            sig: &ev.sig().ok_or(Error::FieldNotFound(MissingField::Sig))?.0,
        })
    }
}

#[cfg(bench)]
mod benches {
    use super::*;
    use crate::test::{black_box, Bencher};

    #[bench]
    pub fn bench_decode_flatbuf_event_borrow(bh: &mut Bencher) {
        let json = r#"{
              "content": "+",
              "created_at": 1716508454,
              "id": "3e9e9c2fbf263590860a9c60a7de6b0d166230a5a15aa8dcdb70f537cec9807a",
              "kind": 7,
              "pubkey": "3bbddb5c7233ad993b41cb639e63122120f391b8580a9b83aae33c648230e0a3",
              "sig": "3f2ba6d713e4851500b81de2d2ef44b72f1eff061898bf8488e74f7e4ed141b0dadab4c3a9c6b237f3a6db83171bd41eafd7ab973f6fb067a4305e95abeadeee",
              "tags": [
                [
                  "e",
                  "e1e786c60ed884b6e784712aaf70e63b848b7403ef651b52b701d87739ea1808",
                  "",
                  "",
                  "04c915daefee38317fa734444acee390a8269fe5810b2241e5e6dd343dfbecc9"
                ],
                [
                  "p",
                  "04c915daefee38317fa734444acee390a8269fe5810b2241e5e6dd343dfbecc9"
                ]
              ]
            }"#;
        let event = Event::from_json(json).unwrap();

        let mut fbb = FlatBufferBuilder::new();
        let bytes = event.encode(&mut fbb);

        bh.iter(|| {
            black_box(EventBorrow::decode(bytes)).unwrap();
        });
    }
}
