// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP98: HTTP Auth
//!
//! This NIP defines an ephemeral event used to authorize requests to HTTP servers using nostr events.
//! This is useful for HTTP services which are build for Nostr and deal with Nostr user accounts.
//!
//! <https://github.com/nostr-protocol/nips/blob/master/98.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use bitcoin::hashes::sha256::Hash as Sha256Hash;

use crate::{Tag, TagStandard, Url};

/// [`HttpData`] required tags
#[derive(Debug, PartialEq, Eq)]
pub enum RequiredTags {
    /// [`TagStandard::AbsoluteURL`]
    AbsoluteURL,
    /// [`TagStandard::Method`]
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

/// NIP98 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Tag missing when parsing
    MissingTag(RequiredTags),
    /// Invalid HTTP Method
    InvalidHttpMethod(String),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTag(tag) => write!(f, "missing tag '{tag}'"),
            Self::InvalidHttpMethod(m) => write!(f, "Invalid HTTP method: {m}"),
        }
    }
}

/// HTTP Method
///
/// <https://github.com/nostr-protocol/nips/blob/master/98.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HttpMethod {
    /// GET
    GET,
    /// POST
    POST,
    /// PUT
    PUT,
    /// PATCH
    PATCH,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::GET => write!(f, "GET"),
            Self::POST => write!(f, "POST"),
            Self::PUT => write!(f, "PUT"),
            Self::PATCH => write!(f, "PATCH"),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "PATCH" => Ok(Self::PATCH),
            m => Err(Error::InvalidHttpMethod(m.to_string())),
        }
    }
}

/// HTTP Data
///
/// <https://github.com/nostr-protocol/nips/blob/master/98.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HttpData {
    /// Absolute request URL
    pub url: Url,
    /// HTTP method
    pub method: HttpMethod,
    /// SHA256 hash of the request body
    pub payload: Option<Sha256Hash>,
}

impl HttpData {
    /// New [`HttpData`]
    #[inline]
    pub fn new(url: Url, method: HttpMethod) -> Self {
        Self {
            url,
            method,
            payload: None,
        }
    }

    /// Add hex-encoded SHA256 hash of the request body
    #[inline]
    pub fn payload(mut self, payload: Sha256Hash) -> Self {
        self.payload = Some(payload);
        self
    }
}

impl From<HttpData> for Vec<Tag> {
    fn from(data: HttpData) -> Self {
        let HttpData {
            url,
            method,
            payload,
        } = data;

        let mut tags: Vec<Tag> = vec![
            Tag::from_standardized_without_cell(TagStandard::AbsoluteURL(url)),
            Tag::from_standardized_without_cell(TagStandard::Method(method)),
        ];
        if let Some(payload) = payload {
            tags.push(Tag::from_standardized_without_cell(TagStandard::Payload(
                payload,
            )));
        }

        tags
    }
}

impl TryFrom<Vec<Tag>> for HttpData {
    type Error = Error;

    fn try_from(value: Vec<Tag>) -> Result<Self, Self::Error> {
        let url = value
            .iter()
            .find_map(|t| match t.as_standardized() {
                Some(TagStandard::AbsoluteURL(u)) => Some(u),
                _ => None,
            })
            .cloned()
            .ok_or(Error::MissingTag(RequiredTags::AbsoluteURL))?;
        let method = value
            .iter()
            .find_map(|t| match t.as_standardized() {
                Some(TagStandard::Method(m)) => Some(m),
                _ => None,
            })
            .cloned()
            .ok_or(Error::MissingTag(RequiredTags::Method))?;
        let payload = value
            .iter()
            .find_map(|t| match t.as_standardized() {
                Some(TagStandard::Payload(p)) => Some(p),
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
