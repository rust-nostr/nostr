// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip90;
use uniffi::Enum;

#[derive(Enum)]
pub enum DataVendingMachineStatus {
    PaymentRequired,
    Processing,
    Error,
    Success,
    Partial,
}

impl From<DataVendingMachineStatus> for nip90::DataVendingMachineStatus {
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
