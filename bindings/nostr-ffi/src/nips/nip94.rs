// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use nostr::hashes::sha256::Hash as Sha256Hash;
use nostr::nips::nip94;
use nostr::Url;
use uniffi::Object;

use crate::error::Result;
use crate::helper::unwrap_or_clone_arc;
use crate::ImageDimensions;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct FileMetadata {
    inner: nip94::FileMetadata,
}

impl Deref for FileMetadata {
    type Target = nip94::FileMetadata;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nip94::FileMetadata> for FileMetadata {
    fn from(inner: nip94::FileMetadata) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl FileMetadata {
    #[uniffi::constructor]
    pub fn new(url: String, mime_type: String, hash: String) -> Result<Self> {
        let url = Url::parse(&url)?;
        let hash = Sha256Hash::from_str(&hash)?;
        Ok(Self {
            inner: nip94::FileMetadata::new(url, mime_type, hash),
        })
    }

    pub fn aes_256_gcm(self: Arc<Self>, key: String, iv: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.aes_256_gcm(key, iv);
        builder
    }

    /// Add file size (bytes)
    pub fn size(self: Arc<Self>, size: u64) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.size(size as usize);
        builder
    }

    /// Add file size (pixels)
    pub fn dimensions(self: Arc<Self>, dim: &ImageDimensions) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.dimensions(**dim);
        builder
    }

    /// Add magnet
    pub fn magnet(self: Arc<Self>, magnet: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.magnet(magnet);
        builder
    }

    /// Add blurhash
    pub fn blurhash(self: Arc<Self>, blurhash: String) -> Self {
        let mut builder = unwrap_or_clone_arc(self);
        builder.inner = builder.inner.blurhash(blurhash);
        builder
    }
}
