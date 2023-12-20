// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use uniffi::Object;

#[derive(Object)]
pub struct ImageDimensions {
    inner: nostr::ImageDimensions,
}

impl From<nostr::ImageDimensions> for ImageDimensions {
    fn from(inner: nostr::ImageDimensions) -> Self {
        Self { inner }
    }
}

impl From<ImageDimensions> for nostr::ImageDimensions {
    fn from(dim: ImageDimensions) -> Self {
        dim.inner
    }
}

impl From<&ImageDimensions> for nostr::ImageDimensions {
    fn from(dim: &ImageDimensions) -> Self {
        dim.inner
    }
}

#[uniffi::export]
impl ImageDimensions {
    #[uniffi::constructor]
    pub fn new(width: u64, height: u64) -> Self {
        Self {
            inner: nostr::ImageDimensions { width, height },
        }
    }

    pub fn width(&self) -> u64 {
        self.inner.width
    }

    pub fn height(&self) -> u64 {
        self.inner.height
    }
}
