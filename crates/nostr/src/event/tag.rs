// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Tag

use bitcoin_hashes::hex::{FromHex, ToHex};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use secp256k1::schnorr::Signature;
use secp256k1::XOnlyPublicKey;
use serde::de::Error as DeserializerError;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use super::id::{self, EventId};
use crate::nips::nip26::Conditions;
use crate::{Kind, Timestamp};

/// [`Tag`] error
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    /// Impossible to parse [`Marker`]
    #[error("impossible to parse marker")]
    MarkerParseError,
    /// Unknown [`Report`]
    #[error("unknown report type")]
    UnknownReportType,
    /// Impossible to find tag kind
    #[error("impossible to find tag kind")]
    KindNotFound,
    /// Invalid length
    #[error("invalid length")]
    InvalidLength,
    /// Impossible to parse integer
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    /// Secp256k1
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    /// Hex decoding error
    #[error(transparent)]
    Hex(#[from] bitcoin_hashes::hex::Error),
    /// Url parse error
    #[error("invalid url")]
    Url(#[from] url::ParseError),
    /// EventId error
    #[error(transparent)]
    EventId(#[from] id::Error),
    /// NIP26 error
    #[error(transparent)]
    Nip26(#[from] crate::nips::nip26::Error),
}

/// Marker
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Marker {
    /// Root
    Root,
    /// Reply
    Reply,
    /// Custom
    Custom(String),
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Root => write!(f, "root"),
            Self::Reply => write!(f, "reply"),
            Self::Custom(m) => write!(f, "{m}"),
        }
    }
}

impl<S> From<S> for Marker
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        let s: String = s.into();
        match s.as_str() {
            "root" => Self::Root,
            "reply" => Self::Reply,
            m => Self::Custom(m.to_string()),
        }
    }
}

/// Report
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Report {
    /// Depictions of nudity, porn, etc
    Nudity,
    /// Profanity, hateful speech, etc.
    Profanity,
    /// Something which may be illegal in some jurisdiction
    ///
    /// Remenber: there is what is right and there is the law.
    Illegal,
    /// Spam
    Spam,
    /// Someone pretending to be someone else
    Impersonation,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nudity => write!(f, "nudity"),
            Self::Profanity => write!(f, "profanity"),
            Self::Illegal => write!(f, "illegal"),
            Self::Spam => write!(f, "spam"),
            Self::Impersonation => write!(f, "impersonation"),
        }
    }
}

impl TryFrom<&str> for Report {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "nudity" => Ok(Self::Nudity),
            "profanity" => Ok(Self::Profanity),
            "illegal" => Ok(Self::Illegal),
            "spam" => Ok(Self::Spam),
            "impersonation" => Ok(Self::Impersonation),
            _ => Err(Error::UnknownReportType),
        }
    }
}

/// Tag kind
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TagKind {
    /// Public key
    P,
    /// Event id
    E,
    /// Reference (URL, etc.)
    R,
    /// Hashtag
    T,
    /// Geohash
    G,
    /// Identifier
    D,
    /// Referencing and tagging
    A,
    /// Relay
    Relay,
    /// Nonce
    Nonce,
    /// Delegation
    Delegation,
    /// Content warning
    ContentWarning,
    /// Expiration
    Expiration,
    /// Subject
    Subject,
    /// Auth challenge
    Challenge,
    /// Title (NIP23)
    Title,
    /// Image (NIP23)
    Image,
    /// Summary (NIP23)
    Summary,
    /// PublishedAt (NIP23)
    PublishedAt,
    /// Bolt11 (NIP57)
    Bolt11,
    /// Preimage (NIP57)
    Preimage,
    /// Description (NIP57)
    Description,
    /// Custom tag kind
    Custom(String),
}

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::P => write!(f, "p"),
            Self::E => write!(f, "e"),
            Self::R => write!(f, "r"),
            Self::T => write!(f, "t"),
            Self::G => write!(f, "g"),
            Self::D => write!(f, "d"),
            Self::A => write!(f, "a"),
            Self::Relay => write!(f, "relay"),
            Self::Nonce => write!(f, "nonce"),
            Self::Delegation => write!(f, "delegation"),
            Self::ContentWarning => write!(f, "content-warning"),
            Self::Expiration => write!(f, "expiration"),
            Self::Subject => write!(f, "subject"),
            Self::Challenge => write!(f, "challenge"),
            Self::Title => write!(f, "title"),
            Self::Image => write!(f, "image"),
            Self::Summary => write!(f, "summary"),
            Self::PublishedAt => write!(f, "published_at"),
            Self::Bolt11 => write!(f, "bolt11"),
            Self::Preimage => write!(f, "preimage"),
            Self::Description => write!(f, "description"),
            Self::Custom(tag) => write!(f, "{tag}"),
        }
    }
}

impl<S> From<S> for TagKind
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        let s: String = s.into();
        match s.as_str() {
            "p" => Self::P,
            "e" => Self::E,
            "r" => Self::R,
            "t" => Self::T,
            "g" => Self::G,
            "d" => Self::D,
            "a" => Self::A,
            "relay" => Self::Relay,
            "nonce" => Self::Nonce,
            "delegation" => Self::Delegation,
            "content-warning" => Self::ContentWarning,
            "expiration" => Self::Expiration,
            "subject" => Self::Subject,
            "challenge" => Self::Challenge,
            "title" => Self::Title,
            "image" => Self::Image,
            "summary" => Self::Summary,
            "published_at" => Self::PublishedAt,
            "bolt11" => Self::Bolt11,
            "preimage" => Self::Preimage,
            "description" => Self::Description,
            tag => Self::Custom(tag.to_string()),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tag {
    Generic(TagKind, Vec<String>),
    Event(EventId, Option<String>, Option<Marker>),
    PubKey(XOnlyPublicKey, Option<String>),
    EventReport(EventId, Report),
    PubKeyReport(XOnlyPublicKey, Report),
    Reference(String),
    RelayMetadata(String, Option<String>),
    Hashtag(String),
    Geohash(String),
    Identifier(String),
    A {
        kind: Kind,
        public_key: XOnlyPublicKey,
        identifier: String,
        relay_url: String,
    },
    Relay(Url),
    ContactList {
        pk: XOnlyPublicKey,
        relay_url: Option<String>,
        alias: Option<String>,
    },
    POW {
        nonce: u128,
        difficulty: u8,
    },
    Delegation {
        delegator_pk: XOnlyPublicKey,
        conditions: Conditions,
        sig: Signature,
    },
    ContentWarning {
        reason: Option<String>,
    },
    Expiration(Timestamp),
    Subject(String),
    Challenge(String),
    Title(String),
    Image(String),
    Summary(String),
    Bolt11(String),
    Preimage(Vec<u8>),
    Description(String),
    PublishedAt(Timestamp),
}

impl Tag {
    /// Parse [`Tag`] from string vector
    pub fn parse<S>(data: Vec<S>) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Tag::try_from(data)
    }

    /// Get [`Tag`] as string vector
    pub fn as_vec(&self) -> Vec<String> {
        self.clone().into()
    }
}

impl<S> TryFrom<Vec<S>> for Tag
where
    S: Into<String>,
{
    type Error = Error;

    fn try_from(tag: Vec<S>) -> Result<Self, Self::Error> {
        let tag: Vec<String> = tag.into_iter().map(|v| v.into()).collect();
        let tag_len: usize = tag.len();
        let tag_kind: TagKind = match tag.first() {
            Some(kind) => TagKind::from(kind),
            None => return Err(Error::KindNotFound),
        };

        if tag_len == 1 {
            match tag_kind {
                TagKind::ContentWarning => Ok(Self::ContentWarning { reason: None }),
                _ => Ok(Self::Generic(tag_kind, Vec::new())),
            }
        } else if tag_len == 2 {
            let content: &str = &tag[1];
            match tag_kind {
                TagKind::P => Ok(Self::PubKey(XOnlyPublicKey::from_str(content)?, None)),
                TagKind::E => Ok(Self::Event(EventId::from_hex(content)?, None, None)),
                TagKind::R => Ok(Self::Reference(content.to_string())),
                TagKind::T => Ok(Self::Hashtag(content.to_string())),
                TagKind::G => Ok(Self::Geohash(content.to_string())),
                TagKind::D => Ok(Self::Identifier(content.to_string())),
                TagKind::Relay => Ok(Self::Relay(Url::parse(content)?)),
                TagKind::ContentWarning => Ok(Self::ContentWarning {
                    reason: Some(content.to_string()),
                }),
                TagKind::Expiration => Ok(Self::Expiration(Timestamp::from_str(content)?)),
                TagKind::Subject => Ok(Self::Subject(content.to_string())),
                TagKind::Challenge => Ok(Self::Challenge(content.to_string())),
                TagKind::Title => Ok(Self::Title(content.to_string())),
                TagKind::Image => Ok(Self::Image(content.to_string())),
                TagKind::Summary => Ok(Self::Summary(content.to_string())),
                TagKind::PublishedAt => Ok(Self::PublishedAt(Timestamp::from_str(content)?)),
                TagKind::Bolt11 => Ok(Self::Bolt11(content.to_string())),
                TagKind::Preimage => Ok(Self::Preimage(Vec::<u8>::from_hex(content)?)),
                TagKind::Description => Ok(Self::Description(content.to_string())),
                _ => Ok(Self::Generic(tag_kind, vec![content.to_string()])),
            }
        } else if tag_len == 3 {
            match tag_kind {
                TagKind::P => {
                    let pubkey = XOnlyPublicKey::from_str(&tag[1])?;
                    if tag[2].is_empty() {
                        Ok(Self::PubKey(pubkey, Some(String::new())))
                    } else {
                        match Report::try_from(tag[2].as_str()) {
                            Ok(report) => Ok(Self::PubKeyReport(pubkey, report)),
                            Err(_) => Ok(Self::PubKey(pubkey, Some(tag[2].clone()))),
                        }
                    }
                }
                TagKind::E => {
                    let event_id = EventId::from_hex(&tag[1])?;
                    if tag[2].is_empty() {
                        Ok(Self::Event(event_id, Some(String::new()), None))
                    } else {
                        match Report::try_from(tag[2].as_str()) {
                            Ok(report) => Ok(Self::EventReport(event_id, report)),
                            Err(_) => Ok(Self::Event(event_id, Some(tag[2].clone()), None)),
                        }
                    }
                }
                TagKind::Nonce => Ok(Self::POW {
                    nonce: tag[1].parse()?,
                    difficulty: tag[2].parse()?,
                }),
                TagKind::A => {
                    let kpi: Vec<&str> = tag[1].split(':').collect();
                    if kpi.len() == 3 {
                        Ok(Self::A {
                            kind: Kind::from_str(kpi[0])?,
                            public_key: XOnlyPublicKey::from_str(kpi[1])?,
                            identifier: kpi[2].to_string(),
                            relay_url: tag[2].clone(),
                        })
                    } else {
                        Err(Error::InvalidLength)
                    }
                }
                _ => Ok(Self::Generic(tag_kind, tag[1..].to_vec())),
            }
        } else if tag_len == 4 {
            match tag_kind {
                TagKind::P => Ok(Self::ContactList {
                    pk: XOnlyPublicKey::from_str(&tag[1])?,
                    relay_url: Some(tag[2].clone()),
                    alias: (!tag[3].is_empty()).then_some(tag[3].clone()),
                }),
                TagKind::E => Ok(Self::Event(
                    EventId::from_hex(&tag[1])?,
                    (!tag[2].is_empty()).then_some(tag[2].clone()),
                    (!tag[3].is_empty()).then_some(Marker::from(&tag[3])),
                )),
                TagKind::Delegation => Ok(Self::Delegation {
                    delegator_pk: XOnlyPublicKey::from_str(&tag[1])?,
                    conditions: Conditions::from_str(&tag[2])?,
                    sig: Signature::from_str(&tag[3])?,
                }),
                _ => Ok(Self::Generic(tag_kind, tag[1..].to_vec())),
            }
        } else {
            Ok(Self::Generic(tag_kind, tag[1..].to_vec()))
        }
    }
}

impl From<Tag> for Vec<String> {
    fn from(data: Tag) -> Self {
        match data {
            Tag::Generic(kind, data) => vec![vec![kind.to_string()], data].concat(),
            Tag::Event(id, relay_url, marker) => {
                let mut tag = vec![TagKind::E.to_string(), id.to_hex()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url);
                }
                if let Some(marker) = marker {
                    if tag.len() == 2 {
                        tag.push(String::new());
                    }
                    tag.push(marker.to_string());
                }
                tag
            }
            Tag::PubKey(pk, relay_url) => {
                let mut tag = vec![TagKind::P.to_string(), pk.to_string()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url);
                }
                tag
            }
            Tag::EventReport(id, report) => {
                vec![TagKind::E.to_string(), id.to_hex(), report.to_string()]
            }
            Tag::PubKeyReport(pk, report) => {
                vec![TagKind::P.to_string(), pk.to_string(), report.to_string()]
            }
            Tag::Reference(r) => vec![TagKind::R.to_string(), r],
            Tag::RelayMetadata(url, rw) => {
                let mut tag = vec![TagKind::R.to_string(), url];
                if let Some(rw) = rw {
                    tag.push(rw);
                }
                tag
            }
            Tag::Hashtag(t) => vec![TagKind::T.to_string(), t],
            Tag::Geohash(g) => vec![TagKind::G.to_string(), g],
            Tag::Identifier(d) => vec![TagKind::D.to_string(), d],
            Tag::A {
                kind,
                public_key,
                identifier,
                relay_url,
            } => vec![
                TagKind::A.to_string(),
                format!("{}:{public_key}:{identifier}", kind.as_u64()),
                relay_url,
            ],
            Tag::Relay(url) => vec![TagKind::Relay.to_string(), url.to_string()],
            Tag::ContactList {
                pk,
                relay_url,
                alias,
            } => vec![
                TagKind::P.to_string(),
                pk.to_string(),
                relay_url.unwrap_or_default(),
                alias.unwrap_or_default(),
            ],
            Tag::POW { nonce, difficulty } => vec![
                TagKind::Nonce.to_string(),
                nonce.to_string(),
                difficulty.to_string(),
            ],
            Tag::Delegation {
                delegator_pk,
                conditions,
                sig,
            } => vec![
                TagKind::Delegation.to_string(),
                delegator_pk.to_string(),
                conditions.to_string(),
                sig.to_string(),
            ],
            Tag::ContentWarning { reason } => {
                let mut tag = vec![TagKind::ContentWarning.to_string()];
                if let Some(reason) = reason {
                    tag.push(reason);
                }
                tag
            }
            Tag::Expiration(timestamp) => {
                vec![TagKind::Expiration.to_string(), timestamp.to_string()]
            }
            Tag::Subject(sub) => vec![TagKind::Subject.to_string(), sub],
            Tag::Challenge(challenge) => vec![TagKind::Challenge.to_string(), challenge],
            Tag::Title(title) => vec![TagKind::Title.to_string(), title],
            Tag::Image(image) => vec![TagKind::Image.to_string(), image],
            Tag::Summary(summary) => vec![TagKind::Summary.to_string(), summary],
            Tag::PublishedAt(timestamp) => {
                vec![TagKind::PublishedAt.to_string(), timestamp.to_string()]
            }
            Tag::Bolt11(bolt11) => vec![TagKind::Bolt11.to_string(), bolt11],
            Tag::Preimage(preimage) => vec![TagKind::Preimage.to_string(), preimage.to_hex()],
            Tag::Description(description) => vec![TagKind::Description.to_string(), description],
        }
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data: Vec<String> = self.as_vec();
        let mut seq = serializer.serialize_seq(Some(data.len()))?;
        for element in data.into_iter() {
            seq.serialize_element(&element)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        type Data = Vec<String>;
        let vec: Vec<String> = Data::deserialize(deserializer)?;
        Self::try_from(vec).map_err(DeserializerError::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Result, Timestamp};

    #[test]
    fn test_deserialize_tag_from_event() -> Result<()> {
        // Got this fresh off the wire
        let event: &str = r#"{"id":"2be17aa3031bdcb006f0fce80c146dea9c1c0268b0af2398bb673365c6444d45","pubkey":"f86c44a2de95d9149b51c6a29afeabba264c18e2fa7c49de93424a0c56947785","created_at":1640839235,"kind":4,"tags":[["p","13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"]],"content":"uRuvYr585B80L6rSJiHocw==?iv=oh6LVqdsYYol3JfFnXTbPA==","sig":"a5d9290ef9659083c490b303eb7ee41356d8778ff19f2f91776c8dc4443388a64ffcf336e61af4c25c05ac3ae952d1ced889ed655b67790891222aaa15b99fdd"}"#;
        let event = Event::from_json(event)?;
        let tag = event.tags.first().unwrap();

        assert_eq!(
            tag,
            &Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                None
            )
        );

        Ok(())
    }

    #[test]
    fn test_serialize_tag_to_event() -> Result<()> {
        let pubkey = XOnlyPublicKey::from_str(
            "68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272",
        )?;
        let event = Event::new_dummy(
            "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
            "79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3",
            Timestamp::from(1671739153),
            4,
            vec![Tag::PubKey(pubkey, None)],
            "8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==",
            "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8"
        )?;

        let event_json: &str = r#"{"content":"8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==","created_at":1671739153,"id":"378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7","kind":4,"pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8","tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"]]}"#;

        assert_eq!(&event.as_json(), event_json);

        Ok(())
    }

    #[test]
    fn test_tag_as_vec() -> Result<()> {
        assert_eq!(
            vec!["content-warning"],
            Tag::ContentWarning { reason: None }.as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ],
            Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                None
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ],
            Tag::Event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                None,
                None
            )
            .as_vec()
        );

        assert_eq!(
            vec!["expiration", "1600000000"],
            Tag::Expiration(Timestamp::from(1600000000)).as_vec()
        );

        assert_eq!(
            vec!["content-warning", "reason"],
            Tag::ContentWarning {
                reason: Some(String::from("reason"))
            }
            .as_vec()
        );

        assert_eq!(
            vec!["subject", "textnote with subject"],
            Tag::Subject(String::from("textnote with subject")).as_vec()
        );

        assert_eq!(
            vec!["client", "nostr-sdk"],
            Tag::Generic(
                TagKind::Custom("client".to_string()),
                vec!["nostr-sdk".to_string()]
            )
            .as_vec()
        );

        assert_eq!(
            vec!["d", "test"],
            Tag::Identifier("test".to_string()).as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ],
            Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                Some(String::from("wss://relay.damus.io"))
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ],
            Tag::Event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                Some(String::new()),
                None
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ],
            Tag::Event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                Some(String::from("wss://relay.damus.io")),
                None
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "spam"
            ],
            Tag::PubKeyReport(
                XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                Report::Spam
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "nudity"
            ],
            Tag::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                Report::Nudity,
            )
            .as_vec()
        );

        assert_eq!(
            vec!["nonce", "1", "20"],
            Tag::POW {
                nonce: 1,
                difficulty: 20
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ],
            Tag::A {
                kind: Kind::LongFormTextNote,
                public_key: XOnlyPublicKey::from_str(
                    "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                )?,
                identifier: String::from("ipsum"),
                relay_url: String::from("wss://relay.nostr.org")
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "alias",
            ],
            Tag::ContactList {
                pk: XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                relay_url: Some(String::from("wss://relay.damus.io")),
                alias: Some(String::from("alias"))
            }
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ],
            Tag::Event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                None,
                Some(Marker::Reply)
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ],
            Tag::Delegation { delegator_pk: XOnlyPublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            )?, conditions: Conditions::from_str("kind=1")?, sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8")? }
            .as_vec()
        );

        Ok(())
    }

    #[test]
    fn test_tag_parser() -> Result<()> {
        assert_eq!(
            Tag::parse::<String>(vec![]).unwrap_err(),
            Error::KindNotFound
        );

        assert_eq!(
            Tag::parse(vec!["content-warning"])?,
            Tag::ContentWarning { reason: None }
        );

        assert_eq!(
            Tag::parse(vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            ])?,
            Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                None
            )
        );

        assert_eq!(
            Tag::parse(vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
            ])?,
            Tag::Event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                None,
                None
            )
        );

        assert_eq!(
            Tag::parse(vec!["expiration", "1600000000"])?,
            Tag::Expiration(Timestamp::from(1600000000))
        );

        assert_eq!(
            Tag::parse(vec!["content-warning", "reason"])?,
            Tag::ContentWarning {
                reason: Some(String::from("reason"))
            }
        );

        assert_eq!(
            Tag::parse(vec!["subject", "textnote with subject"])?,
            Tag::Subject(String::from("textnote with subject"))
        );

        assert_eq!(
            Tag::parse(vec!["client", "nostr-sdk"])?,
            Tag::Generic(
                TagKind::Custom("client".to_string()),
                vec!["nostr-sdk".to_string()]
            )
        );

        assert_eq!(
            Tag::parse(vec!["d", "test"])?,
            Tag::Identifier("test".to_string())
        );

        assert_eq!(
            Tag::parse(vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io"
            ])?,
            Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                Some(String::from("wss://relay.damus.io"))
            )
        );

        assert_eq!(
            Tag::parse(vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                ""
            ])?,
            Tag::Event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                Some(String::new()),
                None
            )
        );

        assert_eq!(
            Tag::parse(vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io"
            ])?,
            Tag::Event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                Some(String::from("wss://relay.damus.io")),
                None
            )
        );

        assert_eq!(
            Tag::parse(vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "impersonation"
            ])?,
            Tag::PubKeyReport(
                XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                Report::Impersonation
            )
        );

        assert_eq!(
            Tag::parse(vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "profanity"
            ])?,
            Tag::EventReport(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                Report::Profanity
            )
        );

        assert_eq!(
            Tag::parse(vec!["nonce", "1", "20"])?,
            Tag::POW {
                nonce: 1,
                difficulty: 20
            }
        );

        assert_eq!(
            Tag::parse(vec![
                "a",
                "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
                "wss://relay.nostr.org"
            ])?,
            Tag::A {
                kind: Kind::LongFormTextNote,
                public_key: XOnlyPublicKey::from_str(
                    "a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919"
                )?,
                identifier: String::from("ipsum"),
                relay_url: String::from("wss://relay.nostr.org")
            }
        );

        assert_eq!(
            Tag::parse(vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io",
                "alias",
            ])?,
            Tag::ContactList {
                pk: XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                relay_url: Some(String::from("wss://relay.damus.io")),
                alias: Some(String::from("alias"))
            }
        );

        assert_eq!(
            Tag::parse(vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "",
                "reply"
            ])?,
            Tag::Event(
                EventId::from_hex(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                None,
                Some(Marker::Reply)
            )
        );

        assert_eq!(
            Tag::parse(vec![
                "delegation",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "kind=1",
                "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8",
            ])?,
            Tag::Delegation { delegator_pk: XOnlyPublicKey::from_str(
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
            )?, conditions: Conditions::from_str("kind=1")?, sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8")? }
        );

        Ok(())
    }
}
