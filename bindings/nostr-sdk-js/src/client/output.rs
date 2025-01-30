// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{HashMap, HashSet};

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::protocol::event::JsEventId;
use crate::relay::JsReconciliation;

#[derive(Clone)]
#[wasm_bindgen(js_name = FailedOutputItem)]
pub struct JsFailedOutputItem {
    #[wasm_bindgen(getter_with_clone)]
    pub url: String,
    #[wasm_bindgen(getter_with_clone)]
    pub error: String,
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

impl From<Output<()>> for JsOutput {
    fn from(value: Output<()>) -> Self {
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
    /// Set of relays that success
    #[wasm_bindgen(getter_with_clone)]
    pub success: Vec<String>,
    /// Map of relays that failed, with related errors.
    #[wasm_bindgen(getter_with_clone)]
    pub failed: Vec<JsFailedOutputItem>,
}

impl From<Output<EventId>> for JsSendEventOutput {
    fn from(output: Output<EventId>) -> Self {
        let out = convert_output(output.success, output.failed);
        Self {
            id: output.val.into(),
            success: out.success,
            failed: out.failed,
        }
    }
}

/// Subscribe output
#[wasm_bindgen(js_name = SubscribeOutput)]
pub struct JsSubscribeOutput {
    /// Subscription ID
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,
    /// Set of relays that success
    #[wasm_bindgen(getter_with_clone)]
    pub success: Vec<String>,
    /// Map of relays that failed, with related errors.
    #[wasm_bindgen(getter_with_clone)]
    pub failed: Vec<JsFailedOutputItem>,
}

impl From<Output<SubscriptionId>> for JsSubscribeOutput {
    fn from(output: Output<SubscriptionId>) -> Self {
        let out = convert_output(output.success, output.failed);
        Self {
            id: output.val.to_string(),
            success: out.success,
            failed: out.failed,
        }
    }
}

/// Reconciliation output
#[wasm_bindgen(js_name = ReconciliationOutput)]
pub struct JsReconciliationOutput {
    #[wasm_bindgen(getter_with_clone)]
    pub report: JsReconciliation,
    /// Set of relays that success
    #[wasm_bindgen(getter_with_clone)]
    pub success: Vec<String>,
    /// Map of relays that failed, with related errors.
    #[wasm_bindgen(getter_with_clone)]
    pub failed: Vec<JsFailedOutputItem>,
}

impl From<Output<Reconciliation>> for JsReconciliationOutput {
    fn from(output: Output<Reconciliation>) -> Self {
        let out = convert_output(output.success, output.failed);
        Self {
            report: output.val.into(),
            success: out.success,
            failed: out.failed,
        }
    }
}

fn convert_output(success: HashSet<RelayUrl>, failed: HashMap<RelayUrl, String>) -> JsOutput {
    JsOutput {
        success: success.into_iter().map(|u| u.to_string()).collect(),
        failed: failed
            .into_iter()
            .map(|(u, e)| JsFailedOutputItem {
                url: u.to_string(),
                error: e,
            })
            .collect(),
    }
}
