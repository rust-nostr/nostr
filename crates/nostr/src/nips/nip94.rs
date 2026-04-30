// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP-94: File Metadata
//!
//! <https://github.com/nostr-protocol/nips/blob/master/94.md>

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

use hashes::hex::HexToArrayError;
use hashes::sha256::Hash as Sha256Hash;

use super::util::{take_and_parse_from_str, take_string};
use crate::event::tag::{Tag, TagCodec, TagCodecError, impl_tag_codec_conversions};
use crate::types::{image, url};
use crate::{ImageDimensions, Url};

const URL: &str = "url";
const MIME_TYPE: &str = "m";
const SHA256: &str = "x";
const ORIGINAL_HASH: &str = "ox";
const SIZE: &str = "size";
const DIMENSIONS: &str = "dim";
const MAGNET: &str = "magnet";
const TORRENT_INFOHASH: &str = "i";
const BLURHASH: &str = "blurhash";
const THUMB: &str = "thumb";
const IMAGE: &str = "image";
const SUMMARY: &str = "summary";
const ALT: &str = "alt";
const FALLBACK: &str = "fallback";
const SERVICE: &str = "service";

/// NIP-94 error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Parse int error
    ParseInt(ParseIntError),
    /// Hex decoding error
    Hex(HexToArrayError),
    /// URL parse error
    Url(url::ParseError),
    /// Image error
    Image(image::Error),
    /// Codec error
    Codec(TagCodecError),
}

impl core::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseInt(e) => e.fmt(f),
            Self::Hex(e) => e.fmt(f),
            Self::Url(e) => e.fmt(f),
            Self::Image(e) => e.fmt(f),
            Self::Codec(e) => e.fmt(f),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

impl From<HexToArrayError> for Error {
    fn from(e: HexToArrayError) -> Self {
        Self::Hex(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<image::Error> for Error {
    fn from(e: image::Error) -> Self {
        Self::Image(e)
    }
}

impl From<TagCodecError> for Error {
    fn from(e: TagCodecError) -> Self {
        Self::Codec(e)
    }
}

/// Potential errors returned when parsing tags into a [`FileMetadata`] struct.
#[derive(Debug, PartialEq, Eq)]
pub enum FileMetadataError {
    /// The URL of the file is missing (no `url` tag)
    MissingUrl,
    /// The mime type of the file is missing (no `m` tag)
    MissingMimeType,
    /// The SHA256 hash of the file is missing (no `x` tag)
    MissingSha,
}

impl core::error::Error for FileMetadataError {}

impl fmt::Display for FileMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingUrl => f.write_str("missing url"),
            Self::MissingMimeType => f.write_str("missing mime type"),
            Self::MissingSha => f.write_str("missing file sha256"),
        }
    }
}

/// Standardized NIP-94 tags
///
/// <https://github.com/nostr-protocol/nips/blob/master/94.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nip94Tag {
    /// `url` tag
    Url(Url),
    /// `m` tag
    MimeType(String),
    /// `x` tag
    Sha256(Sha256Hash),
    /// `ox` tag
    OriginalHash(Sha256Hash),
    /// `size` tag
    Size(usize),
    /// `dim` tag
    Dim(ImageDimensions),
    /// `magnet` tag
    Magnet(String),
    /// `i` tag
    TorrentInfohash(String),
    /// `blurhash` tag
    Blurhash(String),
    /// `thumb` tag
    Thumb {
        /// Thumbnail URL
        url: Url,
        /// Optional SHA256 hash
        hash: Option<Sha256Hash>,
    },
    /// `image` tag
    Image {
        /// Preview image URL
        url: Url,
        /// Optional SHA256 hash
        hash: Option<Sha256Hash>,
    },
    /// `summary` tag
    Summary(String),
    /// `alt` tag
    Alt(String),
    /// `fallback` tag
    Fallback(Url),
    /// `service` tag
    Service(String),
}

impl TagCodec for Nip94Tag {
    type Error = Error;

    fn parse<I, S>(tag: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = tag.into_iter();
        let kind: S = iter.next().ok_or(TagCodecError::missing_tag_kind())?;

        match kind.as_ref() {
            URL => {
                let url: Url = take_and_parse_from_str::<_, _, _, Error>(&mut iter, "URL")?;
                Ok(Self::Url(url))
            }
            MIME_TYPE => Ok(Self::MimeType(take_string(&mut iter, "mime type")?)),
            SHA256 => {
                let hash: Sha256Hash =
                    take_and_parse_from_str::<_, _, _, Error>(&mut iter, "sha256")?;
                Ok(Self::Sha256(hash))
            }
            ORIGINAL_HASH => {
                let hash: Sha256Hash =
                    take_and_parse_from_str::<_, _, _, Error>(&mut iter, "original hash")?;
                Ok(Self::OriginalHash(hash))
            }
            SIZE => {
                let size: usize = take_and_parse_from_str::<_, _, _, Error>(&mut iter, "size")?;
                Ok(Self::Size(size))
            }
            DIMENSIONS => {
                let dim: ImageDimensions =
                    take_and_parse_from_str::<_, _, _, Error>(&mut iter, "dimensions")?;
                Ok(Self::Dim(dim))
            }
            MAGNET => Ok(Self::Magnet(take_string(&mut iter, "magnet link")?)),
            TORRENT_INFOHASH => Ok(Self::TorrentInfohash(take_string(&mut iter, "infohash")?)),
            BLURHASH => Ok(Self::Blurhash(take_string(&mut iter, "blurhash")?)),
            THUMB => {
                let (url, hash) = parse_thumb_or_image_tag(iter, "thumb URL")?;
                Ok(Self::Thumb { url, hash })
            }
            IMAGE => {
                let (url, hash) = parse_thumb_or_image_tag(iter, "image URL")?;
                Ok(Self::Image { url, hash })
            }
            SUMMARY => Ok(Self::Summary(take_string(&mut iter, "summary")?)),
            ALT => Ok(Self::Alt(take_string(&mut iter, "alt")?)),
            FALLBACK => {
                let url: Url =
                    take_and_parse_from_str::<_, _, _, Error>(&mut iter, "fallback URL")?;
                Ok(Self::Fallback(url))
            }
            SERVICE => Ok(Self::Service(take_string(&mut iter, "service name")?)),
            _ => Err(TagCodecError::Unknown.into()),
        }
    }

    fn to_tag(&self) -> Tag {
        match self {
            Self::Url(url) => Tag::new(vec![String::from(URL), url.to_string()]),
            Self::MimeType(mime_type) => {
                Tag::new(vec![String::from(MIME_TYPE), mime_type.to_string()])
            }
            Self::Sha256(hash) => Tag::new(vec![String::from(SHA256), hash.to_string()]),
            Self::OriginalHash(hash) => {
                Tag::new(vec![String::from(ORIGINAL_HASH), hash.to_string()])
            }
            Self::Size(size) => Tag::new(vec![String::from(SIZE), size.to_string()]),
            Self::Dim(dim) => Tag::new(vec![String::from(DIMENSIONS), dim.to_string()]),
            Self::Magnet(uri) => Tag::new(vec![String::from(MAGNET), uri.to_string()]),
            Self::TorrentInfohash(infohash) => {
                Tag::new(vec![String::from(TORRENT_INFOHASH), infohash.to_string()])
            }
            Self::Blurhash(blurhash) => {
                Tag::new(vec![String::from(BLURHASH), blurhash.to_string()])
            }
            Self::Thumb { url, hash } => {
                let mut tag = vec![String::from(THUMB), url.to_string()];
                if let Some(hash) = hash {
                    tag.push(hash.to_string());
                }
                Tag::new(tag)
            }
            Self::Image { url, hash } => {
                let mut tag = vec![String::from(IMAGE), url.to_string()];
                if let Some(hash) = hash {
                    tag.push(hash.to_string());
                }
                Tag::new(tag)
            }
            Self::Summary(summary) => Tag::new(vec![String::from(SUMMARY), summary.to_string()]),
            Self::Alt(alt) => Tag::new(vec![String::from(ALT), alt.to_string()]),
            Self::Fallback(url) => Tag::new(vec![String::from(FALLBACK), url.to_string()]),
            Self::Service(service) => Tag::new(vec![String::from(SERVICE), service.to_string()]),
        }
    }
}

impl_tag_codec_conversions!(Nip94Tag);

/// File Metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileMetadata {
    /// Url
    pub url: Url,
    /// MIME type
    pub mime_type: String,
    /// SHA256 of file
    pub hash: Sha256Hash,
    /// SHA-256 of the original file before any server-side transforms (`ox` tag)
    pub original_hash: Option<Sha256Hash>,
    /// Size in bytes
    pub size: Option<usize>,
    /// Size in pixels
    pub dim: Option<ImageDimensions>,
    /// Magnet
    pub magnet: Option<String>,
    /// Torrent infohash
    pub torrent_infohash: Option<String>,
    /// Blurhash
    pub blurhash: Option<String>,
    /// Thumbnail URL
    pub thumb: Option<Url>,
    /// Thumbnail SHA256 hash
    pub thumb_hash: Option<Sha256Hash>,
    /// Preview image URL
    pub image: Option<Url>,
    /// Preview image SHA256 hash
    pub image_hash: Option<Sha256Hash>,
    /// Short text summary / description
    pub summary: Option<String>,
    /// Alt text for accessibility
    pub alt: Option<String>,
    /// Fallback download URLs
    pub fallback: Vec<Url>,
    /// Serving service type
    pub service: Option<String>,
}

impl FileMetadata {
    /// New [`FileMetadata`]
    pub fn new<S>(url: Url, mime_type: S, hash: Sha256Hash) -> Self
    where
        S: Into<String>,
    {
        Self {
            url,
            mime_type: mime_type.into(),
            hash,
            original_hash: None,
            size: None,
            dim: None,
            magnet: None,
            torrent_infohash: None,
            blurhash: None,
            thumb: None,
            thumb_hash: None,
            image: None,
            image_hash: None,
            summary: None,
            alt: None,
            fallback: Vec::new(),
            service: None,
        }
    }

    /// Set SHA-256 of the original file before server-side transforms (`ox` tag)
    pub fn original_hash(mut self, hash: Sha256Hash) -> Self {
        self.original_hash = Some(hash);
        self
    }

    /// Add file size (bytes)
    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    /// Add file size (pixels)
    pub fn dimensions(mut self, dim: ImageDimensions) -> Self {
        self.dim = Some(dim);
        self
    }

    /// Add magnet
    pub fn magnet<S>(mut self, magnet: S) -> Self
    where
        S: Into<String>,
    {
        self.magnet = Some(magnet.into());
        self
    }

    /// Add torrent infohash
    pub fn torrent_infohash<S>(mut self, infohash: S) -> Self
    where
        S: Into<String>,
    {
        self.torrent_infohash = Some(infohash.into());
        self
    }

    /// Add blurhash
    pub fn blurhash<S>(mut self, blurhash: S) -> Self
    where
        S: Into<String>,
    {
        self.blurhash = Some(blurhash.into());
        self
    }

    /// Add thumbnail URL
    pub fn thumb(mut self, thumb: Url) -> Self {
        self.thumb = Some(thumb);
        self
    }

    /// Add thumbnail SHA256 hash
    pub fn thumb_hash(mut self, hash: Sha256Hash) -> Self {
        self.thumb_hash = Some(hash);
        self
    }

    /// Add preview image URL
    pub fn image(mut self, image: Url) -> Self {
        self.image = Some(image);
        self
    }

    /// Add preview image SHA256 hash
    pub fn image_hash(mut self, hash: Sha256Hash) -> Self {
        self.image_hash = Some(hash);
        self
    }

    /// Add short text summary / description
    pub fn summary<S>(mut self, summary: S) -> Self
    where
        S: Into<String>,
    {
        self.summary = Some(summary.into());
        self
    }

    /// Add alt text for accessibility
    pub fn alt<S>(mut self, alt: S) -> Self
    where
        S: Into<String>,
    {
        self.alt = Some(alt.into());
        self
    }

    /// Add a fallback download URL
    pub fn add_fallback(mut self, url: Url) -> Self {
        self.fallback.push(url);
        self
    }

    /// Add the serving service type
    pub fn service<S>(mut self, service: S) -> Self
    where
        S: Into<String>,
    {
        self.service = Some(service.into());
        self
    }
}

impl From<FileMetadata> for Vec<Tag> {
    fn from(metadata: FileMetadata) -> Self {
        let FileMetadata {
            url,
            mime_type,
            hash,
            original_hash,
            size,
            dim,
            magnet,
            torrent_infohash,
            blurhash,
            thumb,
            thumb_hash,
            image,
            image_hash,
            summary,
            alt,
            fallback,
            service,
        } = metadata;

        let mut tags: Vec<Tag> = Vec::with_capacity(3);

        tags.push(Nip94Tag::Url(url).to_tag());
        tags.push(Nip94Tag::MimeType(mime_type).to_tag());
        tags.push(Nip94Tag::Sha256(hash).to_tag());

        if let Some(hash) = original_hash {
            tags.push(Nip94Tag::OriginalHash(hash).to_tag());
        }

        if let Some(size) = size {
            tags.push(Nip94Tag::Size(size).to_tag());
        }

        if let Some(dim) = dim {
            tags.push(Nip94Tag::Dim(dim).to_tag());
        }

        if let Some(magnet) = magnet {
            tags.push(Nip94Tag::Magnet(magnet).to_tag());
        }

        if let Some(infohash) = torrent_infohash {
            tags.push(Nip94Tag::TorrentInfohash(infohash).to_tag());
        }

        if let Some(blurhash) = blurhash {
            tags.push(Nip94Tag::Blurhash(blurhash).to_tag());
        }

        if let Some(url) = thumb {
            tags.push(
                Nip94Tag::Thumb {
                    url,
                    hash: thumb_hash,
                }
                .to_tag(),
            );
        }

        if let Some(url) = image {
            tags.push(
                Nip94Tag::Image {
                    url,
                    hash: image_hash,
                }
                .to_tag(),
            );
        }

        if let Some(summary) = summary {
            tags.push(Nip94Tag::Summary(summary).to_tag());
        }

        if let Some(alt) = alt {
            tags.push(Nip94Tag::Alt(alt).to_tag());
        }

        for url in fallback {
            tags.push(Nip94Tag::Fallback(url).to_tag());
        }

        if let Some(service) = service {
            tags.push(Nip94Tag::Service(service).to_tag());
        }

        tags
    }
}

impl TryFrom<Vec<Tag>> for FileMetadata {
    type Error = FileMetadataError;

    fn try_from(value: Vec<Tag>) -> Result<Self, Self::Error> {
        let mut url: Option<Url> = None;
        let mut mime_type: Option<String> = None;
        let mut hash: Option<Sha256Hash> = None;
        let mut original_hash: Option<Sha256Hash> = None;
        let mut size: Option<usize> = None;
        let mut dim: Option<ImageDimensions> = None;
        let mut magnet: Option<String> = None;
        let mut torrent_infohash: Option<String> = None;
        let mut blurhash: Option<String> = None;
        let mut thumb: Option<Url> = None;
        let mut thumb_hash: Option<Sha256Hash> = None;
        let mut image: Option<Url> = None;
        let mut image_hash: Option<Sha256Hash> = None;
        let mut summary: Option<String> = None;
        let mut alt: Option<String> = None;
        let mut fallback: Vec<Url> = Vec::new();
        let mut service: Option<String> = None;

        for tag in value.iter().filter_map(|tag| Nip94Tag::try_from(tag).ok()) {
            match tag {
                Nip94Tag::Url(value) => {
                    if url.is_none() {
                        url = Some(value);
                    }
                }
                Nip94Tag::MimeType(value) => {
                    if mime_type.is_none() {
                        mime_type = Some(value);
                    }
                }
                Nip94Tag::Sha256(value) => {
                    if hash.is_none() {
                        hash = Some(value);
                    }
                }
                Nip94Tag::OriginalHash(value) => {
                    if original_hash.is_none() {
                        original_hash = Some(value);
                    }
                }
                Nip94Tag::Size(value) => {
                    if size.is_none() {
                        size = Some(value);
                    }
                }
                Nip94Tag::Dim(value) => {
                    if dim.is_none() {
                        dim = Some(value);
                    }
                }
                Nip94Tag::Magnet(value) => {
                    if magnet.is_none() {
                        magnet = Some(value);
                    }
                }
                Nip94Tag::TorrentInfohash(value) => {
                    if torrent_infohash.is_none() {
                        torrent_infohash = Some(value);
                    }
                }
                Nip94Tag::Blurhash(value) => {
                    if blurhash.is_none() {
                        blurhash = Some(value);
                    }
                }
                Nip94Tag::Thumb { url, hash } => {
                    if thumb.is_none() {
                        thumb = Some(url);
                    }
                    if thumb_hash.is_none() {
                        thumb_hash = hash;
                    }
                }
                Nip94Tag::Image { url, hash } => {
                    if image.is_none() {
                        image = Some(url);
                    }
                    if image_hash.is_none() {
                        image_hash = hash;
                    }
                }
                Nip94Tag::Summary(value) => {
                    if summary.is_none() {
                        summary = Some(value);
                    }
                }
                Nip94Tag::Alt(value) => {
                    if alt.is_none() {
                        alt = Some(value);
                    }
                }
                Nip94Tag::Fallback(value) => fallback.push(value),
                Nip94Tag::Service(value) => {
                    if service.is_none() {
                        service = Some(value);
                    }
                }
            }
        }

        let url = url.ok_or(FileMetadataError::MissingUrl)?;
        let mime_type = mime_type.ok_or(FileMetadataError::MissingMimeType)?;
        let hash = hash.ok_or(FileMetadataError::MissingSha)?;

        let mut metadata = FileMetadata::new(url, mime_type, hash);

        if let Some(hash) = original_hash {
            metadata = metadata.original_hash(hash);
        }

        if let Some(size) = size {
            metadata = metadata.size(size);
        }

        if let Some(dim) = dim {
            metadata = metadata.dimensions(dim);
        }

        if let Some(magnet) = magnet {
            metadata = metadata.magnet(magnet);
        }

        if let Some(infohash) = torrent_infohash {
            metadata = metadata.torrent_infohash(infohash);
        }

        if let Some(blurhash) = blurhash {
            metadata = metadata.blurhash(blurhash);
        }

        if let Some(url) = thumb {
            metadata = metadata.thumb(url);
        }

        if let Some(hash) = thumb_hash {
            metadata = metadata.thumb_hash(hash);
        }

        if let Some(url) = image {
            metadata = metadata.image(url);
        }

        if let Some(hash) = image_hash {
            metadata = metadata.image_hash(hash);
        }

        if let Some(summary) = summary {
            metadata = metadata.summary(summary);
        }

        if let Some(alt) = alt {
            metadata = metadata.alt(alt);
        }

        for url in fallback {
            metadata = metadata.add_fallback(url);
        }

        if let Some(service) = service {
            metadata = metadata.service(service);
        }

        Ok(metadata)
    }
}

fn parse_thumb_or_image_tag<T, S>(
    mut iter: T,
    missing_error: &'static str,
) -> Result<(Url, Option<Sha256Hash>), Error>
where
    T: Iterator<Item = S>,
    S: AsRef<str>,
{
    let url: Url = take_and_parse_from_str::<_, _, _, Error>(&mut iter, missing_error)?;
    let hash: Option<Sha256Hash> = match iter.next() {
        Some(hash) if !hash.as_ref().is_empty() => Some(Sha256Hash::from_str(hash.as_ref())?),
        _ => None,
    };

    Ok((url, hash))
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    const IMAGE_URL: &str = "https://image.nostr.build/99a95fcb4b7a2591ad32467032c52a62d90a204d3b176bc2459ad7427a3f2b89.jpg";
    const IMAGE_HASH: &str = "1aea8e98e0e5d969b7124f553b88dfae47d1f00472ea8c0dbf4ac4577d39ef02";
    const THUMB_URL: &str = "https://image.nostr.build/thumb.jpg";
    const THUMB_HASH: &str = "2aea8e98e0e5d969b7124f553b88dfae47d1f00472ea8c0dbf4ac4577d39ef02";

    #[test]
    fn test_standardized_thumb_tag() {
        let url = Url::parse(THUMB_URL).unwrap();
        let hash = Sha256Hash::from_str(THUMB_HASH).unwrap();
        let tag = vec![String::from("thumb"), url.to_string(), hash.to_string()];
        let parsed = Nip94Tag::parse(&tag).unwrap();

        assert_eq!(
            parsed,
            Nip94Tag::Thumb {
                url: url.clone(),
                hash: Some(hash),
            }
        );
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn test_standardized_service_tag() {
        let tag = vec![String::from("service"), String::from("nip96")];
        let parsed = Nip94Tag::parse(&tag).unwrap();

        assert_eq!(parsed, Nip94Tag::Service(String::from("nip96")));
        assert_eq!(parsed.to_tag(), Tag::parse(tag).unwrap());
    }

    #[test]
    fn parses_valid_tag_vector() {
        let url = Url::parse(IMAGE_URL).unwrap();
        let hash = Sha256Hash::from_str(IMAGE_HASH).unwrap();
        let thumb_url = Url::parse(THUMB_URL).unwrap();
        let thumb_hash = Sha256Hash::from_str(THUMB_HASH).unwrap();
        let dim = ImageDimensions {
            width: 640,
            height: 640,
        };
        let tags = vec![
            Nip94Tag::Dim(dim).to_tag(),
            Nip94Tag::Sha256(hash).to_tag(),
            Nip94Tag::Url(url.clone()).to_tag(),
            Nip94Tag::MimeType(String::from("image/jpeg")).to_tag(),
            Nip94Tag::OriginalHash(hash).to_tag(),
            Nip94Tag::TorrentInfohash(String::from("abc123")).to_tag(),
            Nip94Tag::Thumb {
                url: thumb_url.clone(),
                hash: Some(thumb_hash),
            }
            .to_tag(),
            Nip94Tag::Summary(String::from("example summary")).to_tag(),
            Nip94Tag::Alt(String::from("example alt")).to_tag(),
            Nip94Tag::Fallback(Url::parse("https://fallback.example.com/file.jpg").unwrap())
                .to_tag(),
            Nip94Tag::Service(String::from("nip96")).to_tag(),
        ];
        let got = FileMetadata::try_from(tags).unwrap();
        let expected = FileMetadata::new(url, "image/jpeg", hash)
            .original_hash(hash)
            .dimensions(dim)
            .torrent_infohash("abc123")
            .thumb(thumb_url)
            .thumb_hash(thumb_hash)
            .summary("example summary")
            .alt("example alt")
            .add_fallback(Url::parse("https://fallback.example.com/file.jpg").unwrap())
            .service("nip96");

        assert_eq!(expected, got);
    }

    #[test]
    fn returns_error_with_url_missing() {
        let hash = Sha256Hash::from_str(IMAGE_HASH).unwrap();
        let dim = ImageDimensions {
            width: 640,
            height: 640,
        };
        let tags = vec![
            Nip94Tag::Dim(dim).to_tag(),
            Nip94Tag::Sha256(hash).to_tag(),
            Nip94Tag::MimeType(String::from("image/jpeg")).to_tag(),
        ];
        let got = FileMetadata::try_from(tags).unwrap_err();

        assert_eq!(FileMetadataError::MissingUrl, got);
    }

    #[test]
    fn returns_error_with_mime_type_missing() {
        let url = Url::parse(IMAGE_URL).unwrap();
        let hash = Sha256Hash::from_str(IMAGE_HASH).unwrap();
        let dim = ImageDimensions {
            width: 640,
            height: 640,
        };
        let tags = vec![
            Nip94Tag::Dim(dim).to_tag(),
            Nip94Tag::Sha256(hash).to_tag(),
            Nip94Tag::Url(url).to_tag(),
        ];
        let got = FileMetadata::try_from(tags).unwrap_err();

        assert_eq!(FileMetadataError::MissingMimeType, got);
    }

    #[test]
    fn returns_error_with_sha_missing() {
        let url = Url::parse(IMAGE_URL).unwrap();
        let dim = ImageDimensions {
            width: 640,
            height: 640,
        };
        let tags = vec![
            Nip94Tag::Dim(dim).to_tag(),
            Nip94Tag::Url(url).to_tag(),
            Nip94Tag::MimeType(String::from("image/jpeg")).to_tag(),
        ];
        let got = FileMetadata::try_from(tags).unwrap_err();

        assert_eq!(FileMetadataError::MissingSha, got);
    }
}
