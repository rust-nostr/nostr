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

/// Output
///
/// Send or negentropy reconciliation output
#[derive(Clone)]
#[wasm_bindgen(js_name = Output)]
pub struct JsOutput {
    /// Set of relays that success
    #[wasm_bindgen(getter_with_clone)]
    pub success: Vec<String>,
    /// Map of relays that failed, with related errors.
    #[wasm_bindgen(getter_with_clone)]
    pub failed: Vec<JsFailedOutputItem>,
}

impl From<Output> for JsOutput {
    fn from(value: Output) -> Self {
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
    /// Output
    #[wasm_bindgen(getter_with_clone)]
    pub output: JsOutput,
}

impl From<SendEventOutput> for JsSendEventOutput {
    fn from(value: SendEventOutput) -> Self {
        Self {
            id: value.id.into(),
            output: value.output.into(),
        }
    }
}

/// Subscribe output
#[wasm_bindgen(js_name = SubscribeOutput)]
pub struct JsSubscribeOutput {
    /// Subscription ID
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,
    /// Output
    #[wasm_bindgen(getter_with_clone)]
    pub output: JsOutput,
}

impl From<SubscribeOutput> for JsSubscribeOutput {
    fn from(value: SubscribeOutput) -> Self {
        Self {
            id: value.id.to_string(),
            output: value.output.into(),
        }
    }
}
