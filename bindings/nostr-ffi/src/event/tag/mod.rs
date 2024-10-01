// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::event::tag;
use nostr::{UncheckedUrl, Url};
use uniffi::Object;

pub mod kind;
pub mod standard;

pub use self::kind::TagKind;
pub use self::standard::TagStandard;
use crate::error::Result;
use crate::nips::nip01::Coordinate;
use crate::nips::nip56::Report;
use crate::nips::nip65::RelayMetadata;
use crate::types::filter::SingleLetterTag;
use crate::{EventId, ImageDimensions, PublicKey, Timestamp};

/// Tag
#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct Tag {
    inner: tag::Tag,
}

impl Deref for Tag {
    type Target = tag::Tag;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<tag::Tag> for Tag {
    fn from(inner: tag::Tag) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Tag {
    /// Parse tag
    ///
    /// Return error if the tag is empty!
    #[inline]
    #[uniffi::constructor]
    pub fn parse(data: &[String]) -> Result<Self> {
        Ok(Self {
            inner: tag::Tag::parse(data)?,
        })
    }

    /// Construct from standardized tag
    #[inline]
    #[uniffi::constructor]
    pub fn from_standardized(standardized: TagStandard) -> Result<Self> {
        let standardized: tag::TagStandard = tag::TagStandard::try_from(standardized)?;
        Ok(Self {
            inner: tag::Tag::from_standardized(standardized),
        })
    }

    /// Get tag kind
    #[inline]
    pub fn kind(&self) -> TagKind {
        self.inner.kind().into()
    }

    /// Get tag kind as string
    #[inline]
    pub fn kind_str(&self) -> String {
        self.inner.kind().to_string()
    }

    /// Return the **first** tag value (index `1`), if exists.
    #[inline]
    pub fn content(&self) -> Option<String> {
        self.inner.content().map(|c| c.to_string())
    }

    /// Get `SingleLetterTag`
    #[inline]
    pub fn single_letter_tag(&self) -> Option<Arc<SingleLetterTag>> {
        self.inner.single_letter_tag().map(|s| Arc::new(s.into()))
    }

    /// Get standardized tag
    pub fn as_standardized(&self) -> Option<TagStandard> {
        self.inner.as_standardized().cloned().map(|t| t.into())
    }

    /// Get array of strings
    pub fn as_vec(&self) -> Vec<String> {
        self.inner.as_slice().to_vec()
    }

    /// Compose `["e", "<event-id">]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[uniffi::constructor]
    pub fn event(event_id: &EventId) -> Self {
        Self {
            inner: tag::Tag::event(**event_id),
        }
    }

    /// Compose `["p", "<public-key>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[uniffi::constructor]
    pub fn public_key(public_key: &PublicKey) -> Self {
        Self {
            inner: tag::Tag::public_key(**public_key),
        }
    }

    /// Compose `["d", "<identifier>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[uniffi::constructor]
    pub fn identifier(identifier: &str) -> Self {
        Self {
            inner: tag::Tag::identifier(identifier),
        }
    }

    /// Compose `["a", "<coordinate>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/01.md>
    #[inline]
    #[uniffi::constructor]
    pub fn coordinate(coordinate: &Coordinate) -> Self {
        Self {
            inner: tag::Tag::coordinate(coordinate.deref().clone()),
        }
    }

    /// Compose `["nonce", "<nonce>", "<difficulty>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/13.md>
    #[inline]
    #[uniffi::constructor]
    pub fn pow(nonce: u64, difficulty: u8) -> Self {
        Self {
            inner: tag::Tag::pow(nonce as u128, difficulty),
        }
    }

    /// Compose `["expiration", "<timestamp>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/40.md>
    #[inline]
    #[uniffi::constructor]
    pub fn expiration(timestamp: &Timestamp) -> Self {
        Self {
            inner: tag::Tag::expiration(**timestamp),
        }
    }

    /// Compose `["e", "<event-id>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    #[uniffi::constructor]
    pub fn event_report(event_id: &EventId, report: Report) -> Self {
        Self {
            inner: tag::Tag::event_report(**event_id, report.into()),
        }
    }

    /// Compose `["p", "<public-key>", "<report>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/56.md>
    #[inline]
    #[uniffi::constructor]
    pub fn public_key_report(public_key: &PublicKey, report: Report) -> Self {
        Self {
            inner: tag::Tag::public_key_report(**public_key, report.into()),
        }
    }

    /// Compose `["r", "<relay-url>", "<metadata>"]` tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/65.md>
    #[inline]
    #[uniffi::constructor]
    pub fn relay_metadata(relay_url: &str, metadata: Option<RelayMetadata>) -> Result<Self> {
        let relay_url: Url = Url::from_str(relay_url)?;
        Ok(Self {
            inner: tag::Tag::relay_metadata(relay_url, metadata.map(|m| m.into())),
        })
    }

    /// Compose `["t", "<hashtag>"]` tag
    #[inline]
    #[uniffi::constructor]
    pub fn hashtag(hashtag: &str) -> Self {
        Self {
            inner: tag::Tag::hashtag(hashtag),
        }
    }

    /// Compose `["title", "<title>"]` tag
    #[inline]
    #[uniffi::constructor]
    pub fn title(title: &str) -> Self {
        Self {
            inner: tag::Tag::title(title),
        }
    }

    /// Compose image tag
    #[inline]
    #[uniffi::constructor(default(dimensions = None))]
    pub fn image(url: &str, dimensions: Option<Arc<ImageDimensions>>) -> Self {
        Self {
            inner: tag::Tag::image(UncheckedUrl::from(url), dimensions.map(|d| **d)),
        }
    }

    /// Compose `["description", "<description>"]` tag
    #[inline]
    #[uniffi::constructor]
    pub fn description(description: &str) -> Self {
        Self {
            inner: tag::Tag::description(description),
        }
    }

    /// Protected event
    ///
    /// JSON: `["-"]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    #[inline]
    #[uniffi::constructor]
    pub fn protected() -> Self {
        Self {
            inner: tag::Tag::protected(),
        }
    }

    /// A short human-readable plaintext summary of what that event is about
    ///
    /// JSON: `["alt", "<summary>"]`
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/31.md>
    #[inline]
    #[uniffi::constructor]
    pub fn alt(summary: &str) -> Self {
        Self {
            inner: tag::Tag::alt(summary),
        }
    }

    /// Compose custom tag
    ///
    /// JSON: `["<kind>", "<value-1>", "<value-2>", ...]`
    #[inline]
    #[uniffi::constructor]
    pub fn custom(kind: TagKind, values: &[String]) -> Self {
        Self {
            inner: tag::Tag::custom(kind.into(), values),
        }
    }

    /// Check if is a standard event tag with `root` marker
    pub fn is_root(&self) -> bool {
        self.inner.is_root()
    }

    /// Check if is a standard event tag with `reply` marker
    pub fn is_reply(&self) -> bool {
        self.inner.is_reply()
    }

    /// Check if it's a protected event tag
    ///
    /// <https://github.com/nostr-protocol/nips/blob/master/70.md>
    pub fn is_protected(&self) -> bool {
        self.inner.is_protected()
    }
}
