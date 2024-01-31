// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::{into_err, Result};

#[wasm_bindgen(js_name = Thumbnails)]
pub struct JsThumbnails {
    url: String,
    dimensions: Option<JsImageDimensions>,
}

impl From<JsThumbnails> for (UncheckedUrl, Option<ImageDimensions>) {
    fn from(value: JsThumbnails) -> Self {
        (
            UncheckedUrl::from(value.url),
            value.dimensions.map(|r| r.into()),
        )
    }
}

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
    #[wasm_bindgen]
    pub fn parse(tag: Vec<String>) -> Result<JsTag> {
        Ok(Self {
            inner: Tag::parse(tag).map_err(into_err)?,
        })
    }

    pub fn kind(&self) -> String {
        self.inner.kind().to_string()
    }

    /// Get tag as vector of string
    #[wasm_bindgen(js_name = asVec)]
    pub fn as_vec(&self) -> Vec<String> {
        self.inner.as_vec()
    }

    /// Consume the tag and return vector of string
    #[wasm_bindgen(js_name = toVec)]
    pub fn to_vec(self) -> Vec<String> {
        self.inner.to_vec()
    }
}
