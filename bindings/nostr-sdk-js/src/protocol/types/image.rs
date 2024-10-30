// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Thumbnails)]
pub struct JsThumbnails {
    #[wasm_bindgen(getter_with_clone)]
    pub url: String,
    pub dimensions: Option<JsImageDimensions>,
}

impl From<JsThumbnails> for (UncheckedUrl, Option<ImageDimensions>) {
    fn from(value: JsThumbnails) -> Self {
        (
            UncheckedUrl::from(value.url),
            value.dimensions.map(|r| r.into()),
        )
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
