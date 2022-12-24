// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::XOnlyPublicKey;
use url::Url;

use crate::Sha256Hash;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("impossible to parse marker")]
    MarkerParseError,
    #[error("impossible to find kind")]
    KindNotFound,
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

pub enum TagData {
    Generic(TagKind, Vec<String>),
    EventId(Sha256Hash),
    PubKey(XOnlyPublicKey),
    ContactList {
        pk: XOnlyPublicKey,
        relay_url: String,
        alias: String,
    },
    POW {
        nonce: u128,
        difficulty: u8,
    },
    Nip10E(Sha256Hash, Url, Option<Marker>),
    Delegation {
        delegator_pk: XOnlyPublicKey,
        conditions: String,
        sig: Signature,
    },
    ContentWarning {
        reason: Option<String>,
    },
}

impl From<TagData> for Vec<String> {
    fn from(data: TagData) -> Self {
        match data {
            TagData::Generic(kind, data) => vec![vec![kind.to_string()], data].concat(),
            TagData::EventId(id) => vec![TagKind::E.to_string(), id.to_string()],
            TagData::PubKey(pk) => vec![TagKind::P.to_string(), pk.to_string()],
            TagData::ContactList {
                pk,
                relay_url,
                alias,
            } => vec![TagKind::P.to_string(), pk.to_string(), relay_url, alias],
            TagData::POW { nonce, difficulty } => vec![
                TagKind::Nonce.to_string(),
                nonce.to_string(),
                difficulty.to_string(),
            ],
            TagData::Nip10E(id, relay_url, marker) => {
                let mut tag = vec![
                    TagKind::E.to_string(),
                    id.to_string(),
                    relay_url.to_string(),
                ];
                if let Some(marker) = marker {
                    tag.push(marker.to_string());
                }
                tag
            }
            TagData::Delegation {
                delegator_pk,
                conditions,
                sig,
            } => vec![
                TagKind::Delegation.to_string(),
                delegator_pk.to_string(),
                conditions,
                sig.to_string(),
            ],
            TagData::ContentWarning { reason } => {
                let mut tag = vec![TagKind::ContentWarning.to_string()];
                if let Some(reason) = reason {
                    tag.push(reason);
                }
                tag
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Tag(Vec<String>);

impl From<Vec<String>> for Tag {
    fn from(list: Vec<String>) -> Self {
        Self(list)
    }
}

impl Tag {
    pub fn new(data: TagData) -> Self {
        Self(data.into())
    }

    pub fn kind(&self) -> Result<TagKind, Error> {
        match self.0.first() {
            Some(kind) => Ok(TagKind::from(kind)),
            None => Err(Error::KindNotFound),
        }
    }

    pub fn content(&self) -> Option<&str> {
        self.0.get(1).map(|x| &**x)
    }

    pub fn as_vec(&self) -> Vec<String> {
        self.0.clone()
    }
}
