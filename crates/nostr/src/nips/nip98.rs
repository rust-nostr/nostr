// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP98
//!
//! This NIP defines an ephemerial event used to authorize requests to HTTP servers using nostr events.
//! This is useful for HTTP services which are build for Nostr and deal with Nostr user accounts.
//!
//! <https://github.com/nostr-protocol/nips/blob/master/98.md>

use alloc::vec::Vec;
use core::fmt;

use crate::{HttpMethod, Tag, UncheckedUrl};
use bitcoin::hashes::sha256::Hash as Sha256Hash;

/// [`HttpData`] required tags
#[derive(Debug)]
pub enum RequiredTags {
    /// [`Tag::AbsoluteURL`]
    AbsoluteURL,
    /// [`Tag::Method`]
    Method,
}

impl fmt::Display for RequiredTags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbsoluteURL => write!(f, "url"),
            Self::Method => write!(f, "method"),
        }
    }
}

/// [`HttpData`] error
#[derive(Debug)]
pub enum Error {
    /// Hex decoding error
    Hex(bitcoin::hashes::hex::Error),
    /// Tag missing when parsing
    MissingTag(RequiredTags),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hex(e) => write!(f, "{e}"),
            Self::MissingTag(tag) => write!(f, r#"missing tag "{tag}""#),
        }
    }
}

impl From<bitcoin::hashes::hex::Error> for Error {
    fn from(e: bitcoin::hashes::hex::Error) -> Self {
        Self::Hex(e)
    }
}

/// HTTP Data
pub struct HttpData {
    /// absolute request URL
    pub url: UncheckedUrl,
    /// HTTP method
    pub method: HttpMethod,
    /// SHA256 hash of the request body
    pub payload: Option<Sha256Hash>,
}

impl HttpData {
    /// New [`HttpData`]
    pub fn new(url: UncheckedUrl, method: HttpMethod) -> Self {
        Self {
            url,
            method,
            payload: None,
        }
    }

    /// Add hex-encoded SHA256 hash of the request body
    pub fn payload(self, payload: Sha256Hash) -> Self {
        Self {
            payload: Some(payload),
            ..self
        }
    }
}

impl From<HttpData> for Vec<Tag> {
    fn from(data: HttpData) -> Self {
        let HttpData {
            url,
            method,
            payload,
        } = data;

        let mut tags: Vec<Tag> = vec![Tag::AbsoluteURL(url), Tag::Method(method)];
        if let Some(payload) = payload {
            tags.push(Tag::Payload(payload));
        }

        tags
    }
}

impl TryFrom<Vec<Tag>> for HttpData {
    type Error = Error;

    fn try_from(value: Vec<Tag>) -> Result<Self, Self::Error> {
        let url = value
            .iter()
            .find_map(|t| match t {
                Tag::AbsoluteURL(u) => Some(u),
                _ => None,
            })
            .cloned()
            .ok_or(Error::MissingTag(RequiredTags::AbsoluteURL))?;
        let method = value
            .iter()
            .find_map(|t| match t {
                Tag::Method(m) => Some(m),
                _ => None,
            })
            .cloned()
            .ok_or(Error::MissingTag(RequiredTags::Method))?;
        let payload = value
            .iter()
            .find_map(|t| match t {
                Tag::Payload(p) => Some(p),
                _ => None,
            })
            .cloned();

        Ok(Self {
            url,
            method,
            payload,
        })
    }
}
