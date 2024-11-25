// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
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
    /// Output
    #[wasm_bindgen(getter_with_clone)]
    pub output: JsOutput,
}

impl From<Output<EventId>> for JsSendEventOutput {
    fn from(output: Output<EventId>) -> Self {
        Self {
            id: output.val.into(),
            output: convert_output(output.success, output.failed),
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

impl From<Output<SubscriptionId>> for JsSubscribeOutput {
    fn from(output: Output<SubscriptionId>) -> Self {
        Self {
            id: output.val.to_string(),
            output: convert_output(output.success, output.failed),
        }
    }
}

/// Reconciliation output
#[wasm_bindgen(js_name = ReconciliationOutput)]
pub struct JsReconciliationOutput {
    #[wasm_bindgen(getter_with_clone)]
    pub report: JsReconciliation,
    #[wasm_bindgen(getter_with_clone)]
    pub output: JsOutput,
}

impl From<Output<Reconciliation>> for JsReconciliationOutput {
    fn from(output: Output<Reconciliation>) -> Self {
        Self {
            report: output.val.into(),
            output: convert_output(output.success, output.failed),
        }
    }
}

fn convert_output(
    success: HashSet<RelayUrl>,
    failed: HashMap<RelayUrl, Option<String>>,
) -> JsOutput {
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
