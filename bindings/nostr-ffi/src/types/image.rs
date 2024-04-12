// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use uniffi::Object;

#[derive(Debug, PartialEq, Eq, Hash, Object)]
#[uniffi::export(Debug, Eq, Hash)]
pub struct ImageDimensions {
    inner: nostr::ImageDimensions,
}

impl Deref for ImageDimensions {
    type Target = nostr::ImageDimensions;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<nostr::ImageDimensions> for ImageDimensions {
    fn from(inner: nostr::ImageDimensions) -> Self {
        Self { inner }
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
