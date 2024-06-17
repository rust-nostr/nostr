// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_js::event::JsEventId;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen(js_name = FailedOutputItem)]
pub struct JsFailedOutputItem {
    #[wasm_bindgen(getter_with_clone)]
    pub url: String,
    #[wasm_bindgen(getter_with_clone)]
    pub error: Option<String>,
}

/// Send output
#[wasm_bindgen(js_name = SendOutput)]
pub struct JsSendOutput {
    /// Set of relay urls to which the message/s was successfully sent
    #[wasm_bindgen(getter_with_clone)]
    pub success: Vec<String>,
    /// Map of relay urls with related errors where the message/s wasn't sent
    #[wasm_bindgen(getter_with_clone)]
    pub failed: Vec<JsFailedOutputItem>,
}

impl From<SendOutput> for JsSendOutput {
    fn from(value: SendOutput) -> Self {
        Self {
            success: value.success.into_iter().map(|u| u.to_string()).collect(),
            failed: value
                .failed
                .into_iter()
                .map(|(u, e)| JsFailedOutputItem {
                    url: u.to_string(),
                    error: e,
                })
                .collect(),
        }
    }
}

/// Send event output
#[wasm_bindgen(js_name = SendEventOutput)]
pub struct JsSendEventOutput {
    /// Event ID
    pub id: JsEventId,
    /// Set of relay urls to which the message/s was successfully sent
    #[wasm_bindgen(getter_with_clone)]
    pub success: Vec<String>,
    /// Map of relay urls with related errors where the message/s wasn't sent
    #[wasm_bindgen(getter_with_clone)]
    pub failed: Vec<JsFailedOutputItem>,
}

impl From<SendEventOutput> for JsSendEventOutput {
    fn from(value: SendEventOutput) -> Self {
        Self {
            id: value.id.into(),
            success: value.success.into_iter().map(|u| u.to_string()).collect(),
            failed: value
                .failed
                .into_iter()
                .map(|(u, e)| JsFailedOutputItem {
                    url: u.to_string(),
                    error: e,
                })
                .collect(),
        }
    }
}
