// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! NIP94
//!
//! <https://github.com/nostr-protocol/nips/blob/master/94.md>

use alloc::string::String;
use alloc::vec::Vec;

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use url_fork::Url;

use crate::{ImageDimensions, Tag};

/// File Metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    /// Url
    pub url: Url,
    /// MIME type
    pub mime_type: String,
    /// SHA256 of file
    pub hash: Sha256Hash,
    /// AES 256 GCM
    pub aes_256_gcm: Option<(String, String)>,
    /// Size in bytes
    pub size: Option<usize>,
    /// Size in pixels
    pub dim: Option<ImageDimensions>,
    /// Magnet
    pub magnet: Option<String>,
    /// Blurhash
    pub blurhash: Option<String>,
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
            aes_256_gcm: None,
            size: None,
            dim: None,
            magnet: None,
            blurhash: None,
        }
    }

    /// Add AES 256 GCM
    pub fn aes_256_gcm<S>(self, key: S, iv: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            aes_256_gcm: Some((key.into(), iv.into())),
            ..self
        }
    }

    /// Add file size (bytes)
    pub fn size(self, size: usize) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }

    /// Add file size (pixels)
    pub fn dimensions(self, dim: ImageDimensions) -> Self {
        Self {
            dim: Some(dim),
            ..self
        }
    }

    /// Add magnet
    pub fn magnet<S>(self, magnet: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            magnet: Some(magnet.into()),
            ..self
        }
    }

    /// Add blurhash
    pub fn blurhash<S>(self, blurhash: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            blurhash: Some(blurhash.into()),
            ..self
        }
    }
}

impl From<FileMetadata> for Vec<Tag> {
    fn from(metadata: FileMetadata) -> Self {
        let mut tags = Vec::new();

        let FileMetadata {
            url,
            mime_type,
            hash,
            aes_256_gcm,
            size,
            dim,
            magnet,
            blurhash,
        } = metadata;

        tags.push(Tag::Url(url));
        tags.push(Tag::MimeType(mime_type));
        tags.push(Tag::Sha256(hash));

        if let Some((key, iv)) = aes_256_gcm {
            tags.push(Tag::Aes256Gcm { key, iv });
        }

        if let Some(size) = size {
            tags.push(Tag::Size(size));
        }

        if let Some(dim) = dim {
            tags.push(Tag::Dim(dim));
        }

        if let Some(magnet) = magnet {
            tags.push(Tag::Magnet(magnet));
        }

        if let Some(blurhash) = blurhash {
            tags.push(Tag::Blurhash(blurhash));
        }

        tags
    }
}
