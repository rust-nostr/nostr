// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::Arc;

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

impl From<&ImageDimensions> for nostr::ImageDimensions {
    fn from(dim: &ImageDimensions) -> Self {
        dim.inner
    }
}

#[uniffi::export]
impl ImageDimensions {
    #[uniffi::constructor]
    pub fn new(width: u64, height: u64) -> Arc<Self> {
        Arc::new(Self {
            inner: nostr::ImageDimensions { width, height },
        })
    }

    pub fn width(&self) -> u64 {
        self.inner.width
    }

    pub fn height(&self) -> u64 {
        self.inner.height
    }
}
