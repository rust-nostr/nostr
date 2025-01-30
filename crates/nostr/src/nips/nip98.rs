// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP98: HTTP Auth
//!
//! This NIP defines an ephemeral event used to authorize requests to HTTP servers using nostr events.
//! This is useful for HTTP services which are build for Nostr and deal with Nostr user accounts.
//!
//! <https://github.com/nostr-protocol/nips/blob/master/98.md>

use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

#[cfg(feature = "std")]
use base64::engine::{general_purpose, Engine};
use hashes::sha256::Hash as Sha256Hash;

#[cfg(feature = "std")]
use crate::event::{builder, Event, EventBuilder};
#[cfg(feature = "std")]
use crate::signer::NostrSigner;
#[cfg(feature = "std")]
use crate::util::JsonUtil;
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
    /// Event builder error
    #[cfg(feature = "std")]
    EventBuilder(builder::Error),
    /// Tag missing when parsing
    MissingTag(RequiredTags),
    /// Invalid HTTP Method
    UnknownMethod,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "std")]
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::MissingTag(tag) => write!(f, "missing tag '{tag}'"),
            Self::UnknownMethod => write!(f, "Unknown HTTP method"),
        }
    }
}

#[cfg(feature = "std")]
impl From<builder::Error> for Error {
    fn from(e: builder::Error) -> Self {
        Self::EventBuilder(e)
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
            _ => Err(Error::UnknownMethod),
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

    /// Build the base64-encoded HTTP `Authorization` header **value**.
    ///
    /// Return a string with the following format: `Nostr <base64>`.
    #[cfg(feature = "std")]
    pub async fn to_authorization<T>(self, signer: &T) -> Result<String, Error>
    where
        T: NostrSigner,
    {
        let event: Event = EventBuilder::http_auth(self).sign(signer).await?;
        let encoded: String = general_purpose::STANDARD.encode(event.as_json());
        Ok(format!("Nostr {encoded}"))
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
