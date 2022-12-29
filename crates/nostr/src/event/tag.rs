// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::XOnlyPublicKey;
use serde::de::Error as DeserializerError;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::Sha256Hash;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("impossible to parse marker")]
    MarkerParseError,
    #[error("impossible to find kind")]
    KindNotFound,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("secp256k1 error: {0}")]
    Secp256k1(#[from] bitcoin::secp256k1::Error),
    /// Hex decoding error
    #[error("hex decoding error: {0}")]
    Hex(#[from] bitcoin::hashes::hex::Error),
    /// Url parse error
    #[error("impossible to parse url: {0}")]
    Url(#[from] url::ParseError),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Marker {
    Root,
    Reply,
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Root => write!(f, "root"),
            Self::Reply => write!(f, "reply"),
        }
    }
}

impl FromStr for Marker {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "root" => Ok(Self::Root),
            "reply" => Ok(Self::Reply),
            _ => Err(Error::MarkerParseError),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TagKind {
    P,
    E,
    Nonce,
    Delegation,
    ContentWarning,
    Custom(String),
}

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::P => write!(f, "p"),
            Self::E => write!(f, "e"),
            Self::Nonce => write!(f, "nonce"),
            Self::Delegation => write!(f, "delegation"),
            Self::ContentWarning => write!(f, "content-warning"),
            Self::Custom(tag) => write!(f, "{}", tag),
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
            "nonce" => Self::Nonce,
            "delegation" => Self::Delegation,
            "content-warning" => Self::ContentWarning,
            tag => Self::Custom(tag.to_string()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tag {
    Generic(TagKind, Vec<String>),
    Event(Sha256Hash, Option<Url>, Option<Marker>),
    PubKey(XOnlyPublicKey, Option<Url>),
    ContactList {
        pk: XOnlyPublicKey,
        relay_url: Option<Url>,
        alias: Option<String>,
    },
    POW {
        nonce: u128,
        difficulty: u8,
    },
    Delegation {
        delegator_pk: XOnlyPublicKey,
        conditions: String,
        sig: Signature,
    },
    ContentWarning {
        reason: Option<String>,
    },
}

impl Tag {
    pub fn parse(data: Vec<String>) -> Result<Self, Error> {
        Tag::try_from(data)
    }

    pub fn as_vec(&self) -> Vec<String> {
        self.clone().into()
    }
}

impl TryFrom<Vec<String>> for Tag {
    type Error = Error;

    fn try_from(tag: Vec<String>) -> Result<Self, Self::Error> {
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
                TagKind::E => Ok(Self::Event(Sha256Hash::from_str(content)?, None, None)),
                TagKind::ContentWarning => Ok(Self::ContentWarning {
                    reason: Some(content.to_string()),
                }),
                _ => Ok(Self::Generic(tag_kind, vec![content.to_string()])),
            }
        } else if tag_len == 3 {
            match tag_kind {
                TagKind::P => Ok(Self::PubKey(
                    XOnlyPublicKey::from_str(&tag[1])?,
                    Url::parse(&tag[2]).ok(),
                )),
                TagKind::E => Ok(Self::Event(
                    Sha256Hash::from_str(&tag[1])?,
                    Url::parse(&tag[2]).ok(),
                    None,
                )),
                TagKind::Nonce => Ok(Self::POW {
                    nonce: tag[1].parse()?,
                    difficulty: tag[2].parse()?,
                }),
                _ => Ok(Self::Generic(tag_kind, tag[1..].to_vec())),
            }
        } else if tag_len == 4 {
            match tag_kind {
                TagKind::P => Ok(Self::ContactList {
                    pk: XOnlyPublicKey::from_str(&tag[1])?,
                    relay_url: Url::parse(&tag[2]).ok(),
                    alias: tag[3].is_empty().then_some(tag[3].clone()),
                }),
                TagKind::E => Ok(Self::Event(
                    Sha256Hash::from_str(&tag[1])?,
                    Url::parse(&tag[2]).ok(),
                    Marker::from_str(&tag[3]).ok(),
                )),
                TagKind::Delegation => Ok(Self::Delegation {
                    delegator_pk: XOnlyPublicKey::from_str(&tag[1])?,
                    conditions: tag[2].clone(),
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
                let mut tag = vec![TagKind::E.to_string(), id.to_string()];
                if let Some(relay_url) = relay_url {
                    tag.push(relay_url.to_string());
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
                    tag.push(relay_url.to_string());
                }
                tag
            }
            Tag::ContactList {
                pk,
                relay_url,
                alias,
            } => vec![
                TagKind::P.to_string(),
                pk.to_string(),
                relay_url.map(|u| u.to_string()).unwrap_or_default(),
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
                conditions,
                sig.to_string(),
            ],
            Tag::ContentWarning { reason } => {
                let mut tag = vec![TagKind::ContentWarning.to_string()];
                if let Some(reason) = reason {
                    tag.push(reason);
                }
                tag
            }
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
    use crate::{Event, Result};

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
            1671739153,
            4,
            vec![Tag::PubKey(pubkey, None)],
            "8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==",
            "fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8"
        )?;

        let event_json: &str = r#"{"id":"378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1671739153,"kind":4,"tags":[["p","68d81165918100b7da43fc28f7d1fc12554466e1115886b9e7bb326f65ec4272"]],"content":"8y4MRYrb4ztvXO2NmsHvUA==?iv=MplZo7oSdPfH/vdMC8Hmwg==","sig":"fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8"}"#;

        assert_eq!(&event.as_json()?, event_json);

        Ok(())
    }

    #[test]
    fn test_tag_parse() -> Result<()> {
        assert_eq!(Tag::parse(vec![]).unwrap_err(), Error::KindNotFound);

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
                Sha256Hash::from_str(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                None,
                None
            )
            .as_vec()
        );

        assert_eq!(
            vec!["content-warning", "reason"],
            Tag::ContentWarning {
                reason: Some(String::from("reason"))
            }
            .as_vec()
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
            vec![
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io/"
            ],
            Tag::PubKey(
                XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                Some(Url::parse("wss://relay.damus.io")?)
            )
            .as_vec()
        );

        assert_eq!(
            vec![
                "e",
                "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
                "wss://relay.damus.io/"
            ],
            Tag::Event(
                Sha256Hash::from_str(
                    "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7"
                )?,
                Some(Url::parse("wss://relay.damus.io")?),
                None
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
                "p",
                "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
                "wss://relay.damus.io/",
                "alias",
            ],
            Tag::ContactList {
                pk: XOnlyPublicKey::from_str(
                    "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d"
                )?,
                relay_url: Some(Url::parse("wss://relay.damus.io")?),
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
                Sha256Hash::from_str(
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
            )?, conditions: String::from("kind=1"), sig: Signature::from_str("fd0954de564cae9923c2d8ee9ab2bf35bc19757f8e328a978958a2fcc950eaba0754148a203adec29b7b64080d0cf5a32bebedd768ea6eb421a6b751bb4584a8")? }
            .as_vec()
        );

        Ok(())
    }
}
