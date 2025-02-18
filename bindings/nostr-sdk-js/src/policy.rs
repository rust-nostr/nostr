// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::protocol::event::JsEvent;

#[wasm_bindgen(js_name = AdmitStatus)]
pub enum JsAdmitStatus {
    Success,
    Rejected,
}

impl From<JsAdmitStatus> for AdmitStatus {
    fn from(status: JsAdmitStatus) -> Self {
        match status {
            JsAdmitStatus::Success => Self::Success,
            JsAdmitStatus::Rejected => Self::Rejected,
        }
    }
}

impl TryFrom<JsValue> for JsAdmitStatus {
    type Error = JsValue;

    fn try_from(status: JsValue) -> Result<Self, Self::Error> {
        let status: f64 = status
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Expected AdmitStatus enum value"))?;
        match status as u64 {
            0 => Ok(JsAdmitStatus::Success),
            1 => Ok(JsAdmitStatus::Rejected),
            _ => Err(JsValue::from_str("Unknown AdmitStatus enum value")),
        }
    }
}

#[wasm_bindgen(typescript_custom_section)]
const ADMIT_POLICY: &'static str = r#"
interface AdmitPolicy {
    admitEvent: (event: Event) => Promise<AdmitStatus>;
}
"#;

#[wasm_bindgen]
extern "C" {
    /// Admission policy
    #[wasm_bindgen(typescript_type = "AdmitPolicy", js_name = AdmitPolicy)]
    pub type JsAdmitPolicy;

    /// Admit Event
    ///
    /// Returns `AdmitStatus::Success` if the event is admitted, otherwise `AdmitStatus::Rejected`.
    #[wasm_bindgen(structural, method, js_name = admitEvent)]
    pub async fn admit_event(
        this: &JsAdmitPolicy,
        relay_url: &str,
        subscription_id: &str,
        event: JsEvent,
    ) -> JsValue;
}

pub(crate) struct FFI2RustAdmitPolicy {
    pub(crate) inner: JsAdmitPolicy,
}

unsafe impl Send for FFI2RustAdmitPolicy {}
unsafe impl Sync for FFI2RustAdmitPolicy {}

impl fmt::Debug for FFI2RustAdmitPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FFI2RustAdmitPolicy").finish()
    }
}

mod inner {
    use nostr_sdk::prelude::*;
    use wasm_bindgen::JsValue;

    use super::{FFI2RustAdmitPolicy, JsAdmitStatus};
    use crate::error::MiddleError;

    impl AdmitPolicy for FFI2RustAdmitPolicy {
        fn admit_event<'a>(
            &'a self,
            relay_url: &'a RelayUrl,
            subscription_id: &'a SubscriptionId,
            event: &'a Event,
        ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
            Box::pin(async move {
                let event = event.clone().into();
                let js_value: JsValue = self
                    .inner
                    .admit_event(relay_url.as_str(), subscription_id.as_str(), event)
                    .await;
                let status: JsAdmitStatus = js_value
                    .try_into()
                    .map_err(MiddleError::from)
                    .map_err(PolicyError::backend)?;
                let status: AdmitStatus = status.into();
                Ok(status)
            })
        }
    }
}
