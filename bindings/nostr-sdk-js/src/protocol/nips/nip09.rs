// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use super::nip01::JsCoordinate;
use crate::protocol::event::id::JsEventId;

/// Event deletion request
#[wasm_bindgen(js_name = EventDeletionRequest)]
pub struct JsEventDeletionRequest {
    /// Event IDs
    #[wasm_bindgen(getter_with_clone)]
    pub ids: Vec<JsEventId>,
    /// Event coordinates
    #[wasm_bindgen(getter_with_clone)]
    pub coordinates: Vec<JsCoordinate>,
    /// Optional reason
    #[wasm_bindgen(getter_with_clone)]
    pub reason: Option<String>,
}

impl From<JsEventDeletionRequest> for EventDeletionRequest {
    fn from(request: JsEventDeletionRequest) -> Self {
        Self {
            ids: request.ids.into_iter().map(Into::into).collect(),
            coordinates: request.coordinates.into_iter().map(Into::into).collect(),
            reason: request.reason,
        }
    }
}
