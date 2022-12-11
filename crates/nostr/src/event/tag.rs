// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin_hashes::sha256;
use secp256k1::schnorr::Signature;
use secp256k1::XOnlyPublicKey;
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq)]
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "root" => Ok(Self::Root),
            "reply" => Ok(Self::Reply),
            _ => Err(anyhow!("Impossible to parse marker")),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TagKind {
    P,
    E,
    Nonce,
    Delegation,
}

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::P => write!(f, "p"),
            Self::E => write!(f, "e"),
            Self::Nonce => write!(f, "nonce"),
            Self::Delegation => write!(f, "delegation"),
        }
    }
}

impl FromStr for TagKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "p" => Ok(Self::P),
            "e" => Ok(Self::E),
            "nonce" => Ok(Self::Nonce),
            "delegation" => Ok(Self::Delegation),
            _ => Err(anyhow!("Impossible to parse tag kind")),
        }
    }
}

pub enum TagData {
    Generic(TagKind, Vec<String>),
    EventId(sha256::Hash),
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
    Nip10E(sha256::Hash, Url, Option<Marker>),
    Delegation {
        delegator_pk: XOnlyPublicKey,
        conditions: String,
        sig: Signature,
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
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
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

    pub fn kind(&self) -> Result<TagKind> {
        match self.0.get(0) {
            Some(kind) => Ok(TagKind::from_str(kind)?),
            None => Err(anyhow!("Impossible to find kind")),
        }
    }

    pub fn content(&self) -> Option<&str> {
        self.0.get(1).map(|x| &**x)
    }

    pub fn as_vec(&self) -> Vec<String> {
        self.0.clone()
    }
}
