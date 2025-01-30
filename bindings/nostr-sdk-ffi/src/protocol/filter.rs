// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use nostr::{filter, JsonUtil};
use uniffi::{Enum, Object, Record};

use crate::error::Result;
use crate::protocol::event::{Event, EventId, Kind};
use crate::protocol::key::PublicKey;
use crate::protocol::nips::nip01::Coordinate;
use crate::protocol::types::Timestamp;

#[derive(Enum)]
pub enum Alphabet {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl From<Alphabet> for filter::Alphabet {
    fn from(value: Alphabet) -> Self {
        match value {
            Alphabet::A => Self::A,
            Alphabet::B => Self::B,
            Alphabet::C => Self::C,
            Alphabet::D => Self::D,
            Alphabet::E => Self::E,
            Alphabet::F => Self::F,
            Alphabet::G => Self::G,
            Alphabet::H => Self::H,
            Alphabet::I => Self::I,
            Alphabet::J => Self::J,
            Alphabet::K => Self::K,
            Alphabet::L => Self::L,
            Alphabet::M => Self::M,
            Alphabet::N => Self::N,
            Alphabet::O => Self::O,
            Alphabet::P => Self::P,
            Alphabet::Q => Self::Q,
            Alphabet::R => Self::R,
            Alphabet::S => Self::S,
            Alphabet::T => Self::T,
            Alphabet::U => Self::U,
            Alphabet::V => Self::V,
            Alphabet::W => Self::W,
            Alphabet::X => Self::X,
            Alphabet::Y => Self::Y,
            Alphabet::Z => Self::Z,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct SingleLetterTag {
    inner: filter::SingleLetterTag,
}

impl Deref for SingleLetterTag {
    type Target = filter::SingleLetterTag;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<filter::SingleLetterTag> for SingleLetterTag {
    fn from(inner: filter::SingleLetterTag) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl SingleLetterTag {
    #[uniffi::constructor]
    pub fn lowercase(character: Alphabet) -> Self {
        Self {
            inner: filter::SingleLetterTag::lowercase(character.into()),
        }
    }

    #[uniffi::constructor]
    pub fn uppercase(character: Alphabet) -> Self {
        Self {
            inner: filter::SingleLetterTag::uppercase(character.into()),
        }
    }

    pub fn is_lowercase(&self) -> bool {
        self.inner.is_lowercase()
    }

    pub fn is_uppercase(&self) -> bool {
        self.inner.is_uppercase()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Object)]
#[uniffi::export(Debug, Eq)]
pub struct Filter {
    inner: nostr::Filter,
}

impl Deref for Filter {
    type Target = nostr::Filter;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr::Filter> for Filter {
    fn from(f: nostr::Filter) -> Self {
        Self { inner: f }
    }
}

#[uniffi::export]
impl Filter {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: nostr::Filter::new(),
        }
    }

    pub fn id(&self, id: &EventId) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.id(**id);
        builder
    }

    pub fn ids(&self, ids: &[Arc<EventId>]) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.ids(ids.iter().map(|id| ***id));
        builder
    }

    pub fn remove_ids(&self, ids: &[Arc<EventId>]) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_ids(ids.iter().map(|id| ***id));
        builder
    }

    /// Add event author Public Key
    pub fn author(&self, author: &PublicKey) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.author(**author);
        builder
    }

    pub fn authors(&self, authors: &[Arc<PublicKey>]) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.authors(authors.iter().map(|pk| ***pk));
        builder
    }

    pub fn remove_authors(&self, authors: &[Arc<PublicKey>]) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_authors(authors.iter().map(|pk| ***pk));
        builder
    }

    pub fn kind(&self, kind: &Kind) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.kind(**kind);
        builder
    }

    pub fn kinds(&self, kinds: Vec<Arc<Kind>>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.kinds(kinds.into_iter().map(|k| **k));
        builder
    }

    pub fn remove_kinds(&self, kinds: Vec<Arc<Kind>>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_kinds(kinds.into_iter().map(|k| **k));
        builder
    }

    /// Add event ID (`e` tag)
    pub fn event(&self, event_id: &EventId) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.event(**event_id);
        builder
    }

    /// Add event IDs (`e` tag)
    pub fn events(&self, ids: &[Arc<EventId>]) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.events(ids.iter().map(|id| ***id));
        builder
    }

    pub fn remove_events(&self, ids: &[Arc<EventId>]) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_events(ids.iter().map(|id| ***id));
        builder
    }

    /// Add Public Key (`p` tag)
    pub fn pubkey(&self, pubkey: &PublicKey) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.pubkey(**pubkey);
        builder
    }

    /// Add Public Keys (`p` tag)
    pub fn pubkeys(&self, pubkeys: &[Arc<PublicKey>]) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.pubkeys(pubkeys.iter().map(|pk| ***pk));
        builder
    }

    pub fn remove_pubkeys(&self, pubkeys: &[Arc<PublicKey>]) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_pubkeys(pubkeys.iter().map(|pk| ***pk));
        builder
    }

    pub fn hashtag(&self, hashtag: &str) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.hashtag(hashtag);
        builder
    }

    pub fn hashtags(&self, hashtags: Vec<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.hashtags(hashtags);
        builder
    }

    pub fn remove_hashtags(&self, hashtags: Vec<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_hashtags(hashtags);
        builder
    }

    pub fn reference(&self, reference: &str) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.reference(reference);
        builder
    }

    pub fn references(&self, references: Vec<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.references(references);
        builder
    }

    pub fn remove_references(&self, references: Vec<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_references(references);
        builder
    }

    pub fn identifier(&self, identifier: &str) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.identifier(identifier);
        builder
    }

    pub fn identifiers(&self, identifiers: Vec<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.identifiers(identifiers);
        builder
    }

    pub fn remove_identifiers(&self, identifiers: Vec<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_identifiers(identifiers);
        builder
    }

    /// Add coordinate
    ///
    /// Query for `a` tag.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn coordinate(&self, coordinate: &Coordinate) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.coordinate(coordinate.deref());
        builder
    }

    /// Add coordinates
    ///
    /// Query for `a` tags.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn coordinates(&self, coordinates: Vec<Arc<Coordinate>>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder
            .inner
            .coordinates(coordinates.iter().map(|c| c.as_ref().deref()));
        builder
    }

    /// Remove coordinates
    ///
    /// Remove `a` tags.
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    pub fn remove_coordinates(&self, coordinates: Vec<Arc<Coordinate>>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder
            .inner
            .remove_coordinates(coordinates.iter().map(|c| c.as_ref().deref()));
        builder
    }

    pub fn search(&self, text: &str) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.search(text);
        builder
    }

    pub fn remove_search(&self) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_search();
        builder
    }

    pub fn since(&self, timestamp: &Timestamp) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.since(**timestamp);
        builder
    }

    pub fn remove_since(&self) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_since();
        builder
    }

    pub fn until(&self, timestamp: &Timestamp) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.until(**timestamp);
        builder
    }

    pub fn remove_until(&self) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_until();
        builder
    }

    pub fn limit(&self, limit: u64) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.limit(limit as usize);
        builder
    }

    pub fn remove_limit(&self) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_limit();
        builder
    }

    pub fn custom_tag(&self, tag: &SingleLetterTag, content: String) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.custom_tag(**tag, content);
        builder
    }

    pub fn custom_tags(&self, tag: &SingleLetterTag, contents: Vec<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.custom_tags(**tag, contents);
        builder
    }

    pub fn remove_custom_tags(&self, tag: Arc<SingleLetterTag>, contents: Vec<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.remove_custom_tags(**tag, contents);
        builder
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Determine if `Filter` match given `Event`.
    pub fn match_event(&self, event: &Event) -> bool {
        self.inner.match_event(event.deref())
    }

    #[uniffi::constructor]
    pub fn from_record(record: FilterRecord) -> Self {
        Self {
            inner: record.into(),
        }
    }

    pub fn as_record(&self) -> FilterRecord {
        self.inner.clone().into()
    }

    #[uniffi::constructor]
    pub fn from_json(json: String) -> Result<Self> {
        Ok(Self {
            inner: nostr::Filter::from_json(json)?,
        })
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(self.inner.try_as_json()?)
    }
}

#[derive(Record)]
pub struct GenericTag {
    pub key: Arc<SingleLetterTag>,
    pub value: Vec<String>,
}

#[derive(Record)]
pub struct FilterRecord {
    pub ids: Option<Vec<Arc<EventId>>>,
    pub authors: Option<Vec<Arc<PublicKey>>>,
    pub kinds: Option<Vec<Arc<Kind>>>,
    /// It's a string describing a query in a human-readable form, i.e. "best nostr apps"
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/50.md>
    pub search: Option<String>,
    /// An integer unix timestamp, events must be newer than this to pass
    pub since: Option<Arc<Timestamp>>,
    /// An integer unix timestamp, events must be older than this to pass
    pub until: Option<Arc<Timestamp>>,
    /// Maximum number of events to be returned in the initial query
    pub limit: Option<u64>,
    /// Generic tag queries
    pub generic_tags: Vec<GenericTag>,
}

impl From<nostr::Filter> for FilterRecord {
    fn from(f: nostr::Filter) -> Self {
        Self {
            ids: f
                .ids
                .map(|ids| ids.into_iter().map(|v| Arc::new(v.into())).collect()),
            authors: f
                .authors
                .map(|authors| authors.into_iter().map(|v| Arc::new(v.into())).collect()),
            kinds: f
                .kinds
                .map(|kinds| kinds.into_iter().map(|v| Arc::new(v.into())).collect()),
            search: f.search,
            since: f.since.map(|t| Arc::new(t.into())),
            until: f.until.map(|t| Arc::new(t.into())),
            limit: f.limit.map(|l| l as u64),
            generic_tags: f
                .generic_tags
                .into_iter()
                .map(|(k, v)| GenericTag {
                    key: Arc::new(k.into()),
                    value: v.into_iter().map(|v| v.to_string()).collect(),
                })
                .collect(),
        }
    }
}

impl From<FilterRecord> for nostr::Filter {
    fn from(f: FilterRecord) -> Self {
        Self {
            ids: f.ids.map(|ids| ids.into_iter().map(|v| **v).collect()),
            authors: f
                .authors
                .map(|authors| authors.into_iter().map(|v| **v).collect()),
            kinds: f
                .kinds
                .map(|kinds| kinds.into_iter().map(|v| **v).collect()),
            search: f.search,
            since: f.since.map(|t| **t),
            until: f.until.map(|t| **t),
            limit: f.limit.map(|l| l as usize),
            generic_tags: f
                .generic_tags
                .into_iter()
                .map(|GenericTag { key, value }| (**key, value.into_iter().collect()))
                .collect(),
        }
    }
}
