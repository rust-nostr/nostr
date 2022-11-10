// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin_hashes::sha256;
use secp256k1::XOnlyPublicKey;

use super::kind::KindBase;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TagKind {
    P,
    E,
    Nonce,
}

impl fmt::Display for TagKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::P => write!(f, "p"),
            Self::E => write!(f, "e"),
            Self::Nonce => write!(f, "nonce"),
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
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Tag(Vec<String>);

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

    pub fn parse(&self, kind_base: KindBase) -> Result<TagData> {
        if let KindBase::ContactList = kind_base {
            if let Some(pk) = self.0.get(1) {
                let pk = XOnlyPublicKey::from_str(pk)?;
                let relay_url = self.0.get(2).cloned();
                let alias = self.0.get(3).cloned();
                return Ok(TagData::ContactList {
                    pk,
                    relay_url: relay_url.unwrap_or_default(),
                    alias: alias.unwrap_or_default(),
                });
            }
        }

        Err(anyhow!("Impossible to parse tag"))
    }
}
