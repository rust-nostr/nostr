// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP65: Relay List Metadata
//!
//! <https://github.com/nostr-protocol/nips/blob/master/65.md>

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::nips::util::take_relay_url;
use crate::types::url;
use crate::{Event, RelayUrl};

const RELAY_METADATA: &str = "r";

/// NIP56 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Url error
    Url(url::Error),
    /// Codec error
    Codec(TagCodecError),
    /// Invalid Relay Metadata
    InvalidRelayMetadata,
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Url(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
            Self::InvalidRelayMetadata => f.write_str("Invalid relay metadata"),
        }
    }
}

impl From<url::Error> for Error {
    fn from(e: url::Error) -> Self {
        Self::Url(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Relay Metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelayMetadata {
    /// Read
    Read,
    /// Write
    Write,
}

impl RelayMetadata {
    /// Get as `&str`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }

    /// Check if is [RelayMetadata::Read]
    #[inline]
    pub fn is_read(&self) -> bool {
        matches!(self, Self::Read)
    }

    /// Check if is [RelayMetadata::Write]
    #[inline]
    pub fn is_write(&self) -> bool {
        matches!(self, Self::Write)
    }
}

impl fmt::Display for RelayMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RelayMetadata {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            _ => Err(Error::InvalidRelayMetadata),
        }
    }
}

/// Standardized NIP-65 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/65.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip65Tag {
    /// Relay metadata
    RelayMetadata {
        /// Relay URL
        relay_url: RelayUrl,
        /// Relay metadata
        metadata: Option<RelayMetadata>,
    },
}

impl TagCodec for Nip65Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            RELAY_METADATA => {
                let relay_url: RelayUrl = take_relay_url::<_, _, Error>(&mut iter)?;

                let metadata: Option<RelayMetadata> = match iter.next() {
                    Some(metadata) => Some(RelayMetadata::from_str(metadata.as_ref())?),
                    None => None,
                };

                Ok(Self::RelayMetadata {
                    relay_url,
                    metadata,
                })
            }
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::RelayMetadata {
                relay_url,
                metadata,
            } => {
                let mut tag: Vec<String> = Vec::with_capacity(2 + metadata.is_some() as usize);

                tag.push(String::from(RELAY_METADATA));
                tag.push(relay_url.to_string());

                if let Some(metadata) = metadata {
                    tag.push(metadata.to_string());
                }

                Tag::new(tag)
            }
        }
    }
}

impl_tag_codec_conversions!(Nip65Tag);

/// Extracts the relay info (url, optional read/write flag) from the event
#[inline]
pub fn extract_relay_list(
    event: &Event,
) -> impl Iterator<Item = (RelayUrl, Option<RelayMetadata>)> + '_ {
    event
        .tags
        .iter()
        .filter_map(|tag| match Nip65Tag::try_from(tag) {
            Ok(Nip65Tag::RelayMetadata {
                relay_url,
                metadata,
            }) => Some((relay_url, metadata)),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standardized_relay_metadata_tag() {
        let relay_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let tag = vec!["r".to_string(), relay_url.to_string(), String::from("read")];
        let parsed = Nip65Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip65Tag::RelayMetadata {
                relay_url: relay_url.clone(),
                metadata: Some(RelayMetadata::Read),
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_standardized_relay_metadata_tag_without_marker() {
        let relay_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let tag = vec!["r".to_string(), relay_url.to_string()];
        let parsed = Nip65Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip65Tag::RelayMetadata {
                relay_url: relay_url.clone(),
                metadata: None,
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }
}
