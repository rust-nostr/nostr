// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

pub mod list;

pub use self::list::JsTags;
use super::JsEventId;
use crate::error::{into_err, Result};
use crate::protocol::filter::JsSingleLetterTag;
use crate::protocol::key::JsPublicKey;
use crate::protocol::nips::nip01::JsCoordinate;
use crate::protocol::nips::nip56::JsReport;
use crate::protocol::nips::nip65::JsRelayMetadata;
use crate::protocol::types::image::JsImageDimensions;
use crate::protocol::types::JsTimestamp;

/// Tag
#[wasm_bindgen(js_name = Tag)]
pub struct JsTag {
    pub(crate) inner: Tag,
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
            inner: Tag::parse(tag).map_err(into_err)?,
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
    #[wasm_bindgen(js_name = singleLetterTag)]
    pub fn single_letter_tag(&self) -> Option<JsSingleLetterTag> {
        self.inner.single_letter_tag().map(|s| s.into())
    }

    /// Get tag len
    pub fn len(&self) -> u64 {
        self.inner.len() as u64
    }

    /// Get array of strings
    #[inline]
    #[wasm_bindgen(js_name = asVec)]
    pub fn as_vec(&self) -> Vec<String> {
        self.inner.as_slice().to_vec()
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

    /// Construct `["client", "<name>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/89.md>
    #[inline]
    pub fn client(name: String) -> Self {
        Self {
            inner: Tag::client(name),
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
    #[wasm_bindgen(js_name = eventReport)]
    pub fn event_report(event_id: &JsEventId, report: JsReport) -> Self {
        Self {
            inner: Tag::event_report(**event_id, report.into()),
        }
    }

    /// Compose `["p", "<public-key>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    #[wasm_bindgen(js_name = publicKeyReport)]
    pub fn public_key_report(public_key: &JsPublicKey, report: JsReport) -> Self {
        Self {
            inner: Tag::public_key_report(**public_key, report.into()),
        }
    }

    /// Compose `["r", "<relay-url>", "<metadata>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[inline]
    #[wasm_bindgen(js_name = relayMetadata)]
    pub fn relay_metadata(relay_url: &str, metadata: Option<JsRelayMetadata>) -> Result<JsTag> {
        let relay_url: RelayUrl = RelayUrl::parse(relay_url).map_err(into_err)?;
        Ok(Self {
            inner: Tag::relay_metadata(relay_url, metadata.map(|m| m.into())),
        })
    }

    /// Compose `["t", "<hashtag>"]` tag
    ///
    /// This will convert the hashtag to lowercase.
    #[inline]
    pub fn hashtag(hashtag: &str) -> Self {
        Self {
            inner: Tag::hashtag(hashtag),
        }
    }

    /// Compose `["r", "<value>"]` tag
    #[inline]
    pub fn reference(reference: &str) -> Self {
        Self {
            inner: Tag::reference(reference),
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
    pub fn image(url: &str, dimensions: Option<JsImageDimensions>) -> Result<JsTag> {
        Ok(Self {
            inner: Tag::image(
                Url::parse(url).map_err(into_err)?,
                dimensions.map(|d| d.into()),
            ),
        })
    }

    /// Compose `["description", "<description>"]` tag
    #[inline]
    pub fn description(description: &str) -> Self {
        Self {
            inner: Tag::description(description),
        }
    }

    /// Protected event
    ///
    /// JSON: `["-"]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    #[inline]
    pub fn protected() -> Self {
        Self {
            inner: Tag::protected(),
        }
    }

    /// A short human-readable plaintext summary of what that event is about
    ///
    /// JSON: `["alt", "<summary>"]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    #[inline]
    pub fn alt(summary: &str) -> Self {
        Self {
            inner: Tag::alt(summary),
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

    /// Check if it's a protected event tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    #[inline]
    #[wasm_bindgen(js_name = isProtected)]
    pub fn is_protected(&self) -> bool {
        self.inner.is_protected()
    }
}
