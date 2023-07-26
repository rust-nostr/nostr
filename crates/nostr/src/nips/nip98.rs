// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP98
//!
//! This NIP defines an ephemerial event used to authorize requests to HTTP servers using nostr events.
//! This is useful for HTTP services which are build for Nostr and deal with Nostr user accounts.
//! <https://github.com/nostr-protocol/nips/blob/master/98.md>
use url::Url;

use crate::Tag;

/// HTTP Data
pub struct HttpData {
    /// absolute request URL
    pub url: Url,
    /// HTTP method
    pub method: String,
    /// SHA256 hash of the request body
    pub payload: Option<String>,
}

impl HttpData {
    /// New [`HttpData`]
    pub fn new<S>(url: Url, method: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            url,
            method: method.into(),
            payload: None,
        }
    }

    /// Add hex-encoded SHA256 hash of the request body
    pub fn payload<S>(self, payload: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            payload: Some(payload.into()),
            ..self
        }
    }
}

impl From<HttpData> for Vec<Tag> {
    fn from(value: HttpData) -> Self {
        let mut tags = Vec::new();

        let HttpData {
            url,
            method,
            payload,
        } = value;

        tags.push(Tag::HttpAuthUrl(url));
        tags.push(Tag::Method(method));

        if let Some(payload) = payload {
            tags.push(Tag::Payload(payload))
        }

        tags
    }
}
