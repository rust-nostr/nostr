// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use core::ops::Deref;

use js_sys::Function;
use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::protocol::event::JsEvent;

#[wasm_bindgen(js_name = Events)]
pub struct JsEvents {
    inner: Events,
}

impl From<Events> for JsEvents {
    fn from(inner: Events) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = Events)]
impl JsEvents {
    /// Returns the number of events in the collection.
    pub fn len(&self) -> u64 {
        self.inner.len() as u64
    }

    /// Returns the number of events in the collection.
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if contains `Event`
    pub fn contains(&self, event: &JsEvent) -> bool {
        self.inner.contains(event.deref())
    }

    /// Merge events collections into a single one.
    ///
    /// Collection is converted to unbounded if one of the merge `Events` have a different hash.
    /// In other words, the filters limit is respected only if the `Events` are related to the same
    /// list of filters.
    pub fn merge(self, other: JsEvents) -> Self {
        self.inner.merge(other.inner).into()
    }

    /// Get first `Event` (descending order)
    pub fn first(&self) -> Option<JsEvent> {
        self.inner.first().cloned().map(|e| e.into())
    }

    /// Convert collection to vector of events.
    #[wasm_bindgen(js_name = forEach)]
    pub fn for_each(&self, callbackfn: EventsForEach) {
        self.inner.iter().cloned().for_each(|e| {
            let event: JsEvent = e.into();
            callbackfn.call1(&JsValue::NULL, &event.into()).unwrap();
        });
    }

    /// Convert collection to vector of events.
    #[wasm_bindgen(js_name = toVec)]
    pub fn to_vec(self) -> Vec<JsEvent> {
        self.inner.into_iter().map(|e| e.into()).collect()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Function, is_type_of = JsValue::is_function, typescript_type = "(event: Event) => void")]
    pub type EventsForEach;
}
