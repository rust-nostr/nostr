// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip90::DataVendingMachineStatus;
use wasm_bindgen::prelude::*;

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
