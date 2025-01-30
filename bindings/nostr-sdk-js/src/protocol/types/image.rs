// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::error::into_err;

#[wasm_bindgen(js_name = Thumbnails)]
pub struct JsThumbnails {
    #[wasm_bindgen(getter_with_clone)]
    pub url: String,
    pub dimensions: Option<JsImageDimensions>,
}

impl TryFrom<JsThumbnails> for (Url, Option<ImageDimensions>) {
    type Error = JsValue;

    fn try_from(value: JsThumbnails) -> Result<Self, Self::Error> {
        Ok((
            Url::parse(&value.url).map_err(into_err)?,
            value.dimensions.map(|r| r.into()),
        ))
    }
}

#[wasm_bindgen(js_class = Thumbnails)]
impl JsThumbnails {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, dimensions: Option<JsImageDimensions>) -> Self {
        Self { url, dimensions }
    }
}

#[derive(Clone, Copy)]
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
