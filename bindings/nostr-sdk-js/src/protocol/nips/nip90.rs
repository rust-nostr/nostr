// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

use crate::protocol::event::JsEvent;

#[wasm_bindgen(js_name = DataVendingMachineStatus)]
pub enum JsDataVendingMachineStatus {
    PaymentRequired,
    Processing,
    Error,
    Success,
    Partial,
}

impl From<DataVendingMachineStatus> for JsDataVendingMachineStatus {
    fn from(value: DataVendingMachineStatus) -> Self {
        match value {
            DataVendingMachineStatus::PaymentRequired => Self::PaymentRequired,
            DataVendingMachineStatus::Processing => Self::Processing,
            DataVendingMachineStatus::Error => Self::Error,
            DataVendingMachineStatus::Success => Self::Success,
            DataVendingMachineStatus::Partial => Self::Partial,
        }
    }
}

impl From<JsDataVendingMachineStatus> for DataVendingMachineStatus {
    fn from(value: JsDataVendingMachineStatus) -> Self {
        match value {
            JsDataVendingMachineStatus::PaymentRequired => Self::PaymentRequired,
            JsDataVendingMachineStatus::Processing => Self::Processing,
            JsDataVendingMachineStatus::Error => Self::Error,
            JsDataVendingMachineStatus::Success => Self::Success,
            JsDataVendingMachineStatus::Partial => Self::Partial,
        }
    }
}

/// Data Vending Machine (DVM) - Job Feedback data
///
/// <https://github.com/nostr-protocol/nips/blob/master/90.md>
#[wasm_bindgen(js_name = JobFeedbackData)]
pub struct JsJobFeedbackData {
    inner: JobFeedbackData,
}

impl Deref for JsJobFeedbackData {
    type Target = JobFeedbackData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<JobFeedbackData> for JsJobFeedbackData {
    fn from(inner: JobFeedbackData) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen(js_class = JobFeedbackData)]
impl JsJobFeedbackData {
    /// Construct new Job Feedback
    #[wasm_bindgen(constructor)]
    pub fn new(job_request: &JsEvent, status: JsDataVendingMachineStatus) -> Self {
        Self {
            inner: JobFeedbackData::new(job_request.deref(), status.into()),
        }
    }

    /// Add extra info
    pub fn extra_info(self, info: String) -> Self {
        self.inner.extra_info(info).into()
    }

    /// Add payment amount
    pub fn amount(self, millisats: u64, bolt11: Option<String>) -> Self {
        self.inner.amount(millisats, bolt11).into()
    }

    /// Add payload
    pub fn payload(self, payload: String) -> Self {
        self.inner.payload(payload).into()
    }
}
