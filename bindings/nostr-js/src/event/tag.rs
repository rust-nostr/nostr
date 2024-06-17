// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};
use crate::event::JsEventId;
use crate::key::JsPublicKey;
use crate::nips::nip01::JsCoordinate;
use crate::types::filter::JsSingleLetterTag;
use crate::types::JsTimestamp;

/// Report
///
/// <https://github.com/nostr-protocol/nips/blob/master/56.md>
#[wasm_bindgen(js_name = Report)]
pub enum JsReport {
    /// Depictions of nudity, porn, etc
    Nudity,
    /// Profanity, hateful speech, etc.
    Profanity,
    /// Something which may be illegal in some jurisdiction
    Illegal,
    /// Spam
    Spam,
    /// Someone pretending to be someone else
    Impersonation,
    ///  Reports that don't fit in the above categories
    Other,
}

impl From<JsReport> for Report {
    fn from(value: JsReport) -> Self {
        match value {
            JsReport::Nudity => Self::Nudity,
            JsReport::Profanity => Self::Profanity,
            JsReport::Illegal => Self::Illegal,
            JsReport::Spam => Self::Spam,
            JsReport::Impersonation => Self::Impersonation,
            JsReport::Other => Self::Other,
        }
    }
}

#[wasm_bindgen(js_name = HttpMethod)]
pub enum JsHttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
}

impl From<HttpMethod> for JsHttpMethod {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::GET => Self::GET,
            HttpMethod::POST => Self::POST,
            HttpMethod::PUT => Self::PUT,
            HttpMethod::PATCH => Self::PATCH,
        }
    }
}

impl From<JsHttpMethod> for HttpMethod {
    fn from(value: JsHttpMethod) -> Self {
        match value {
            JsHttpMethod::GET => Self::GET,
            JsHttpMethod::POST => Self::POST,
            JsHttpMethod::PUT => Self::PUT,
            JsHttpMethod::PATCH => Self::PATCH,
        }
    }
}

#[wasm_bindgen(js_name = Thumbnails)]
pub struct JsThumbnails {
    #[wasm_bindgen(getter_with_clone)]
    pub url: String,
    pub dimensions: Option<JsImageDimensions>,
}

impl From<JsThumbnails> for (UncheckedUrl, Option<ImageDimensions>) {
    fn from(value: JsThumbnails) -> Self {
        (
            UncheckedUrl::from(value.url),
            value.dimensions.map(|r| r.into()),
        )
    }
}

#[wasm_bindgen(js_class = Thumbnails)]
impl JsThumbnails {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, dimensions: Option<JsImageDimensions>) -> Self {
        Self { url, dimensions }
    }
}

#[derive(Clone, Copy)]
#[wasm_bindgen(js_name = ImageDimensions)]
pub struct JsImageDimensions {
    pub width: u64,
    pub height: u64,
}

impl From<ImageDimensions> for JsImageDimensions {
    fn from(value: ImageDimensions) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<JsImageDimensions> for ImageDimensions {
    fn from(value: JsImageDimensions) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(Clone, Copy)]
#[wasm_bindgen(js_name = RelayMetadata)]
pub enum JsRelayMetadata {
    Read,
    Write,
}

impl From<RelayMetadata> for JsRelayMetadata {
    fn from(value: RelayMetadata) -> Self {
        match value {
            RelayMetadata::Read => Self::Read,
            RelayMetadata::Write => Self::Write,
        }
    }
}

impl From<JsRelayMetadata> for RelayMetadata {
    fn from(value: JsRelayMetadata) -> Self {
        match value {
            JsRelayMetadata::Read => Self::Read,
            JsRelayMetadata::Write => Self::Write,
        }
    }
}

/// Tag
#[wasm_bindgen(js_name = Tag)]
pub struct JsTag {
    inner: Tag,
}

impl From<Tag> for JsTag {
    fn from(inner: Tag) -> Self {
        Self { inner }
    }
}

impl From<JsTag> for Tag {
    fn from(tag: JsTag) -> Self {
        tag.inner
    }
}

#[wasm_bindgen(js_class = Tag)]
impl JsTag {
    /// Parse tag
    ///
    /// Return error if the tag is empty!
    #[inline]
    #[wasm_bindgen]
    pub fn parse(tag: Vec<String>) -> Result<JsTag> {
        Ok(Self {
            inner: Tag::parse(&tag).map_err(into_err)?,
        })
    }

    /// Get tag kind
    #[inline]
    pub fn kind(&self) -> String {
        self.inner.kind().to_string()
    }

    /// Return the **first** tag value (index `1`), if exists.
    #[inline]
    pub fn content(&self) -> Option<String> {
        self.inner.content().map(|c| c.to_string())
    }

    /// Get `SingleLetterTag`
    #[inline]
    pub fn single_letter_tag(&self) -> Option<JsSingleLetterTag> {
        self.inner.single_letter_tag().map(|s| s.into())
    }

    /// Get array of strings
    #[inline]
    #[wasm_bindgen(js_name = asVec)]
    pub fn as_vec(&self) -> Vec<String> {
        self.inner.as_vec().to_vec()
    }

    /// Consume tag and return array of strings
    #[inline]
    #[wasm_bindgen(js_name = toVec)]
    pub fn to_vec(self) -> Vec<String> {
        self.inner.to_vec()
    }

    /// Compose `["e", "<event-id">]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn event(event_id: &JsEventId) -> Self {
        Self {
            inner: Tag::event(**event_id),
        }
    }

    /// Compose `["p", "<public-key>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(public_key: &JsPublicKey) -> Self {
        Self {
            inner: Tag::public_key(**public_key),
        }
    }

    /// Compose `["d", "<identifier>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn identifier(identifier: &str) -> Self {
        Self {
            inner: Tag::identifier(identifier),
        }
    }

    /// Compose `["a", "<coordinate>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    pub fn coordinate(coordinate: &JsCoordinate) -> Self {
        Self {
            inner: Tag::coordinate(coordinate.deref().clone()),
        }
    }

    /// Compose `["nonce", "<nonce>", "<difficulty>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    #[inline]
    pub fn pow(nonce: u64, difficulty: u8) -> Self {
        Self {
            inner: Tag::pow(nonce as u128, difficulty),
        }
    }

    /// Compose `["expiration", "<timestamp>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[inline]
    pub fn expiration(timestamp: &JsTimestamp) -> Self {
        Self {
            inner: Tag::expiration(**timestamp),
        }
    }

    /// Compose `["e", "<event-id>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    pub fn event_report(event_id: &JsEventId, report: JsReport) -> Self {
        Self {
            inner: Tag::event_report(**event_id, report.into()),
        }
    }

    /// Compose `["p", "<public-key>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    pub fn public_key_report(public_key: &JsPublicKey, report: JsReport) -> Self {
        Self {
            inner: Tag::public_key_report(**public_key, report.into()),
        }
    }

    /// Compose `["r", "<relay-url>", "<metadata>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[inline]
    pub fn relay_metadata(relay_url: &str, metadata: Option<JsRelayMetadata>) -> Result<JsTag> {
        let relay_url: Url = Url::parse(relay_url).map_err(into_err)?;
        Ok(Self {
            inner: Tag::relay_metadata(relay_url, metadata.map(|m| m.into())),
        })
    }

    /// Compose `["t", "<hashtag>"]` tag
    #[inline]
    pub fn hashtag(hashtag: &str) -> Self {
        Self {
            inner: Tag::hashtag(hashtag),
        }
    }

    /// Compose `["title", "<title>"]` tag
    #[inline]
    pub fn title(title: &str) -> Self {
        Self {
            inner: Tag::title(title),
        }
    }

    /// Compose image tag
    #[inline]
    pub fn image(url: &str, dimensions: Option<JsImageDimensions>) -> Self {
        Self {
            inner: Tag::image(UncheckedUrl::from(url), dimensions.map(|d| d.into())),
        }
    }

    /// Compose `["description", "<description>"]` tag
    #[inline]
    pub fn description(description: &str) -> Self {
        Self {
            inner: Tag::description(description),
        }
    }

    /// Check if is a standard event tag with `root` marker
    #[inline]
    #[wasm_bindgen(js_name = isRoot)]
    pub fn is_root(&self) -> bool {
        self.inner.is_root()
    }

    /// Check if is a standard event tag with `reply` marker
    #[inline]
    #[wasm_bindgen(js_name = isReply)]
    pub fn is_reply(&self) -> bool {
        self.inner.is_reply()
    }
}
