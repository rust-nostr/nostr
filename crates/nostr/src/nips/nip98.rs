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
use hashes::Hash;

#[cfg(feature = "std")]
use crate::event::{self, builder, Event, EventBuilder};
#[cfg(feature = "std")]
use crate::signer::NostrSigner;
#[cfg(feature = "std")]
use crate::util::JsonUtil;
#[cfg(feature = "std")]
use crate::{Kind, PublicKey, TagKind, Timestamp};
use crate::{Tag, TagStandard, Url};

#[cfg(feature = "std")]
const AUTH_HEADER_PREFIX: &str = "Nostr";

/// [`HttpData`] required tags
#[derive(Debug, PartialEq, Eq)]
pub enum RequiredTags {
    /// [`TagStandard::AbsoluteURL`]
    AbsoluteURL,
    /// [`TagStandard::Method`]
    Method,
    /// [`TagStandard::Payload`]
    Payload,
}

impl fmt::Display for RequiredTags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbsoluteURL => write!(f, "u"),
            Self::Method => write!(f, "method"),
            Self::Payload => write!(f, "payload"),
        }
    }
}

/// NIP98 error
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Base64 error
    #[cfg(feature = "std")]
    Base64(base64::DecodeError),
    /// Event error
    #[cfg(feature = "std")]
    Event(event::Error),
    /// Event builder error
    #[cfg(feature = "std")]
    EventBuilder(builder::Error),
    /// Tag missing when parsing
    MissingTag(RequiredTags),
    /// Invalid HTTP Method
    UnknownMethod,
    /// Nostr authorization header missing
    #[cfg(feature = "std")]
    AuthorizationHeaderMissing,
    /// Malformed authorization header
    #[cfg(feature = "std")]
    MalformedAuthorizationHeader,
    /// Unexpected authorization header kind
    #[cfg(feature = "std")]
    WrongAuthHeaderKind,
    /// Authorization doesn't match request
    #[cfg(feature = "std")]
    AuthorizationNotMatchRequest {
        /// The authorized url
        authorized_url: Box<Url>,
        /// The authorized url
        authorized_method: HttpMethod,
        /// The request url
        request_url: Box<Url>,
        /// The request url
        request_method: HttpMethod,
    },
    /// Authorization is too old
    #[cfg(feature = "std")]
    AuthorizationTooOld {
        /// Current timestamp
        current: Timestamp,
        /// Auth event created at
        created_at: Timestamp,
    },
    /// Payload hash doesn't match the body hash
    #[cfg(feature = "std")]
    PayloadHashMismatch,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "std")]
            Self::Base64(e) => write!(f, "{e}"),
            #[cfg(feature = "std")]
            Self::Event(e) => write!(f, "{e}"),
            #[cfg(feature = "std")]
            Self::EventBuilder(e) => write!(f, "{e}"),
            Self::MissingTag(tag) => write!(f, "missing '{tag}' tag"),
            Self::UnknownMethod => write!(f, "Unknown HTTP method"),
            #[cfg(feature = "std")]
            Self::AuthorizationHeaderMissing => write!(f, "nostr authorization header missing"),
            #[cfg(feature = "std")]
            Self::MalformedAuthorizationHeader => write!(f, "malformed nostr authorization header"),
            #[cfg(feature = "std")]
            Self::WrongAuthHeaderKind => write!(f, "wrong nostr authorization header kind"),
            #[cfg(feature = "std")]
            Self::AuthorizationNotMatchRequest { authorized_url, authorized_method, request_url, request_method} => write!(f, "authorization doesn't match request: authorized_url={authorized_url}, authorized_method={authorized_method}, request_url={request_url}, request_method={request_method}"),
            #[cfg(feature = "std")]
            Self::AuthorizationTooOld { current, created_at } => write!(f, "authorization event is too old: current_time={current}, created_at={created_at}"),
            #[cfg(feature = "std")]
            Self::PayloadHashMismatch => write!(f, "payload hash doesn't match the body hash"),
        }
    }
}

#[cfg(feature = "std")]
impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64(e)
    }
}

#[cfg(feature = "std")]
impl From<event::Error> for Error {
    fn from(e: event::Error) -> Self {
        Self::Event(e)
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
        Ok(format!("{AUTH_HEADER_PREFIX} {encoded}"))
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

/// Verify authentication header
///
/// Returns the event [`PublicKey`] if the authorization is valid.
///
/// This functions execute the following checks:
/// - Extract the prefix and the base64 event from the header;
/// - Decode the base64 event and check if the kind is correct;
/// - Check if the tags are right;
/// - Check if the auth event is too old
/// - If there is a body, verify if the payload hash matches the body hash;
/// - Verify the event ID and signature (to learn more check [`Event::verify`]).
///
/// <https://github.com/nostr-protocol/nips/blob/master/98.md>
#[cfg(feature = "std")]
pub fn verify_auth_header(
    auth_header: &str,
    url: &Url,
    method: HttpMethod,
    current_time: Timestamp,
    body: Option<&[u8]>,
) -> Result<PublicKey, Error> {
    // Original code at https://github.com/damus-io/notepush/blob/63c5f7e7236f7bfe09f665b5fb4a03b412284d13/src/nip98_auth.rs

    if auth_header.is_empty() {
        return Err(Error::AuthorizationHeaderMissing);
    }

    let (prefix, base64_encoded_event): (&str, &str) = auth_header
        .split_once(' ')
        .ok_or(Error::MalformedAuthorizationHeader)?;

    if prefix != AUTH_HEADER_PREFIX || base64_encoded_event.is_empty() {
        return Err(Error::MalformedAuthorizationHeader);
    }

    // Decode event
    let decoded_event_json: Vec<u8> = general_purpose::STANDARD.decode(base64_encoded_event)?;
    let event: Event = Event::from_json(decoded_event_json)?;

    // Check event kind
    if event.kind != Kind::HttpAuth {
        return Err(Error::WrongAuthHeaderKind);
    }

    let authorized_url: &Url = event
        .tags
        .find_standardized(TagKind::u())
        .and_then(|tag| match tag {
            TagStandard::AbsoluteURL(u) => Some(u),
            _ => None,
        })
        .ok_or(Error::MissingTag(RequiredTags::AbsoluteURL))?;

    let authorized_method: &HttpMethod = event
        .tags
        .find_standardized(TagKind::Method)
        .and_then(|tag| match tag {
            TagStandard::Method(u) => Some(u),
            _ => None,
        })
        .ok_or(Error::MissingTag(RequiredTags::Method))?;

    if authorized_url != url || authorized_method != &method {
        return Err(Error::AuthorizationNotMatchRequest {
            authorized_url: Box::new(authorized_url.clone()),
            authorized_method: *authorized_method,
            request_url: Box::new(url.clone()),
            request_method: method,
        });
    }

    let time_delta = TimeDelta::subtracting(current_time, event.created_at);
    if (time_delta.negative && time_delta.delta_abs_seconds > 30)
        || (!time_delta.negative && time_delta.delta_abs_seconds > 60)
    {
        return Err(Error::AuthorizationTooOld {
            current: current_time,
            created_at: event.created_at,
        });
    }

    if let Some(body_data) = body {
        // Get payload hash
        let payload: &Sha256Hash = match event.tags.find_standardized(TagKind::Payload) {
            Some(TagStandard::Payload(p)) => p,
            _ => return Err(Error::MissingTag(RequiredTags::Payload)),
        };

        // Hash body data
        let body_hash: Sha256Hash = Sha256Hash::hash(body_data);

        // Check if payload and body hash matches
        if payload != &body_hash {
            return Err(Error::PayloadHashMismatch);
        }
    }

    // Verify both the Event ID and the cryptographic signature
    event.verify()?;

    Ok(event.pubkey)
}

#[cfg(feature = "std")]
struct TimeDelta {
    pub delta_abs_seconds: u64,
    pub negative: bool,
}

#[cfg(feature = "std")]
impl TimeDelta {
    /// Safely calculate the difference between two timestamps in seconds
    /// This function is safer against overflows than subtracting the timestamps directly
    pub fn subtracting(t1: Timestamp, t2: Timestamp) -> TimeDelta {
        if t1 > t2 {
            TimeDelta {
                delta_abs_seconds: (t1 - t2).as_u64(),
                negative: false,
            }
        } else {
            TimeDelta {
                delta_abs_seconds: (t2 - t1).as_u64(),
                negative: true,
            }
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn empty_auth_header() {
        let url = Url::parse("https://example.com/").unwrap();
        assert_eq!(
            verify_auth_header("", &url, HttpMethod::GET, Timestamp::now(), None).unwrap_err(),
            Error::AuthorizationHeaderMissing
        );
    }

    #[test]
    fn malformed_auth_header() {
        let url = Url::parse("https://example.com/").unwrap();
        let now = Timestamp::now();
        assert_eq!(
            verify_auth_header("Test Nostr", &url, HttpMethod::GET, now, None).unwrap_err(),
            Error::MalformedAuthorizationHeader
        );
        assert_eq!(
            verify_auth_header("Nostr", &url, HttpMethod::GET, now, None).unwrap_err(),
            Error::MalformedAuthorizationHeader
        );
        assert_eq!(verify_auth_header("nostr eyJpZCI6ImZlOTY0ZTc1ODkwMzM2MGYyOGQ4NDI0ZDA5MmRhODQ5NGVkMjA3Y2JhODIzMTEwYmUzYTU3ZGZlNGI1Nzg3MzQiLCJwdWJrZXkiOiI2M2ZlNjMxOGRjNTg1ODNjZmUxNjgxMGY4NmRkMDllMThiZmQ3NmFhYmMyNGEwMDgxY2UyODU2ZjMzMDUwNGVkIiwiY29udGVudCI6IiIsImtpbmQiOjI3MjM1LCJjcmVhdGVkX2F0IjoxNjgyMzI3ODUyLCJ0YWdzIjpbWyJ1IiwiaHR0cHM6Ly9hcGkuc25vcnQuc29jaWFsL2FwaS92MS9uNXNwL2xpc3QiXSxbIm1ldGhvZCIsIkdFVCJdXSwic2lnIjoiNWVkOWQ4ZWM5NThiYzg1NGY5OTdiZGMyNGFjMzM3ZDAwNWFmMzcyMzI0NzQ3ZWZlNGEwMGUyNGY0YzMwNDM3ZmY0ZGQ4MzA4Njg0YmVkNDY3ZDlkNmJlM2U1YTUxN2JiNDNiMTczMmNjN2QzMzk0OWEzYWFmODY3MDVjMjIxODQifQ==", &url, HttpMethod::GET, now, None).unwrap_err(), Error::MalformedAuthorizationHeader);
    }

    #[test]
    fn auth_header_wrong_kind() {
        let url = Url::parse("https://example.com/").unwrap();
        let now = Timestamp::now();
        assert_eq!(verify_auth_header("Nostr eyJpZCI6ImZlOTY0ZTc1ODkwMzM2MGYyOGQ4NDI0ZDA5MmRhODQ5NGVkMjA3Y2JhODIzMTEwYmUzYTU3ZGZlNGI1Nzg3MzQiLCJwdWJrZXkiOiI2M2ZlNjMxOGRjNTg1ODNjZmUxNjgxMGY4NmRkMDllMThiZmQ3NmFhYmMyNGEwMDgxY2UyODU2ZjMzMDUwNGVkIiwiY29udGVudCI6IiIsImtpbmQiOjEsImNyZWF0ZWRfYXQiOjE2ODIzMjc4NTIsInRhZ3MiOltbInUiLCJodHRwczovL2FwaS5zbm9ydC5zb2NpYWwvYXBpL3YxL241c3AvbGlzdCJdLFsibWV0aG9kIiwiR0VUIl1dLCJzaWciOiI1ZWQ5ZDhlYzk1OGJjODU0Zjk5N2JkYzI0YWMzMzdkMDA1YWYzNzIzMjQ3NDdlZmU0YTAwZTI0ZjRjMzA0MzdmZjRkZDgzMDg2ODRiZWQ0NjdkOWQ2YmUzZTVhNTE3YmI0M2IxNzMyY2M3ZDMzOTQ5YTNhYWY4NjcwNWMyMjE4NCJ9", &url, HttpMethod::GET, now, None).unwrap_err(), Error::WrongAuthHeaderKind);
    }

    #[test]
    fn auth_header_not_match_request() {
        let url = Url::parse("https://example.com/").unwrap(); // Expected url: https://api.snort.social/api/v1/n5sp/list
        let now = Timestamp::now();
        let method = HttpMethod::POST;
        assert_eq!(verify_auth_header("Nostr eyJpZCI6ImZlOTY0ZTc1ODkwMzM2MGYyOGQ4NDI0ZDA5MmRhODQ5NGVkMjA3Y2JhODIzMTEwYmUzYTU3ZGZlNGI1Nzg3MzQiLCJwdWJrZXkiOiI2M2ZlNjMxOGRjNTg1ODNjZmUxNjgxMGY4NmRkMDllMThiZmQ3NmFhYmMyNGEwMDgxY2UyODU2ZjMzMDUwNGVkIiwiY29udGVudCI6IiIsImtpbmQiOjI3MjM1LCJjcmVhdGVkX2F0IjoxNjgyMzI3ODUyLCJ0YWdzIjpbWyJ1IiwiaHR0cHM6Ly9hcGkuc25vcnQuc29jaWFsL2FwaS92MS9uNXNwL2xpc3QiXSxbIm1ldGhvZCIsIkdFVCJdXSwic2lnIjoiNWVkOWQ4ZWM5NThiYzg1NGY5OTdiZGMyNGFjMzM3ZDAwNWFmMzcyMzI0NzQ3ZWZlNGEwMGUyNGY0YzMwNDM3ZmY0ZGQ4MzA4Njg0YmVkNDY3ZDlkNmJlM2U1YTUxN2JiNDNiMTczMmNjN2QzMzk0OWEzYWFmODY3MDVjMjIxODQifQ==", &url, method, now, None).unwrap_err(), Error::AuthorizationNotMatchRequest {
            authorized_url: Box::new(Url::parse("https://api.snort.social/api/v1/n5sp/list").unwrap()),
            authorized_method: HttpMethod::GET,
            request_url: Box::new(url),
            request_method: HttpMethod::POST,
        });
    }

    #[test]
    fn auth_header_too_old() {
        let url = Url::parse("https://api.snort.social/api/v1/n5sp/list").unwrap();
        let method = HttpMethod::GET;
        let now = Timestamp::from_secs(1777777777);
        assert_eq!(verify_auth_header("Nostr eyJpZCI6ImZlOTY0ZTc1ODkwMzM2MGYyOGQ4NDI0ZDA5MmRhODQ5NGVkMjA3Y2JhODIzMTEwYmUzYTU3ZGZlNGI1Nzg3MzQiLCJwdWJrZXkiOiI2M2ZlNjMxOGRjNTg1ODNjZmUxNjgxMGY4NmRkMDllMThiZmQ3NmFhYmMyNGEwMDgxY2UyODU2ZjMzMDUwNGVkIiwiY29udGVudCI6IiIsImtpbmQiOjI3MjM1LCJjcmVhdGVkX2F0IjoxNjgyMzI3ODUyLCJ0YWdzIjpbWyJ1IiwiaHR0cHM6Ly9hcGkuc25vcnQuc29jaWFsL2FwaS92MS9uNXNwL2xpc3QiXSxbIm1ldGhvZCIsIkdFVCJdXSwic2lnIjoiNWVkOWQ4ZWM5NThiYzg1NGY5OTdiZGMyNGFjMzM3ZDAwNWFmMzcyMzI0NzQ3ZWZlNGEwMGUyNGY0YzMwNDM3ZmY0ZGQ4MzA4Njg0YmVkNDY3ZDlkNmJlM2U1YTUxN2JiNDNiMTczMmNjN2QzMzk0OWEzYWFmODY3MDVjMjIxODQifQ==", &url, method, now, None).unwrap_err(), Error::AuthorizationTooOld {
            current: now,
            created_at: Timestamp::from_secs(1682327852),
        });
    }

    #[test]
    fn valid_auth_header() {
        let url = Url::parse("https://example.com").unwrap();
        let method = HttpMethod::GET;
        let now = Timestamp::from_secs(1742462605);
        let public_key =
            PublicKey::from_hex("aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4")
                .unwrap();
        assert_eq!(verify_auth_header("Nostr eyJpZCI6IjEyYjQ2YmUwMDg5MjI3OWU3YmJhYThlYTg5ODA5ZWNiMWYyYzk5MmY1ZDk0ZWRmMmNkYmQ2Y2JhNmVlMzBjMDMiLCJwdWJrZXkiOiJhYTRmYzg2NjVmNTY5NmUzM2RiN2UxYTU3MmUzYjBmNWIzZDYxNTgzN2IwZjM2MmRjYjFjODA2OGIwOThjN2I0IiwiY3JlYXRlZF9hdCI6MTc0MjQ2MjYwNSwia2luZCI6MjcyMzUsInRhZ3MiOltbInUiLCJodHRwczovL2V4YW1wbGUuY29tLyJdLFsibWV0aG9kIiwiR0VUIl1dLCJjb250ZW50IjoiIiwic2lnIjoiZWEzNGU3NDA3ZGQ2OTFjNDJhYzY3ZjQ3YTMwYjBmMDEwZTFiYWYwMjM3MjhiNzI4OGFlYzA0Zjg3MzMyYmZlYTRhZjJkNDdiNTJiMjhkNGMxMGMwOWY3NmNiZGFhNWZjMTE0OTNiOTlkZTU2NDhmMzlhM2JkYzMwYjQxMTNjNjMifQ==", &url, method, now, None).unwrap(), public_key);
    }
}
