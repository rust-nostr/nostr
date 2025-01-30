// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::ops::Deref;

use nostr::nips::nip90;
use uniffi::{Enum, Object};

use crate::protocol::event::Event;

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

impl From<nip90::DataVendingMachineStatus> for DataVendingMachineStatus {
    fn from(value: nip90::DataVendingMachineStatus) -> Self {
        match value {
            nip90::DataVendingMachineStatus::PaymentRequired => Self::PaymentRequired,
            nip90::DataVendingMachineStatus::Processing => Self::Processing,
            nip90::DataVendingMachineStatus::Error => Self::Error,
            nip90::DataVendingMachineStatus::Success => Self::Success,
            nip90::DataVendingMachineStatus::Partial => Self::Partial,
        }
    }
}

/// Data Vending Machine (DVM) - Job Feedback data
///
/// <https://github.com/nostr-protocol/nips/blob/master/90.md>
#[derive(Clone, Object)]
pub struct JobFeedbackData {
    inner: nip90::JobFeedbackData,
}

impl Deref for JobFeedbackData {
    type Target = nip90::JobFeedbackData;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl JobFeedbackData {
    /// Construct new Job Feedback
    #[uniffi::constructor]
    pub fn new(job_request: &Event, status: DataVendingMachineStatus) -> Self {
        Self {
            inner: nip90::JobFeedbackData::new(job_request.deref(), status.into()),
        }
    }

    /// Add extra info
    pub fn extra_info(&self, info: String) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.extra_info(info);
        builder
    }

    /// Add payment amount
    pub fn amount(&self, millisats: u64, bolt11: Option<String>) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.amount(millisats, bolt11);
        builder
    }

    /// Add payload
    pub fn payload(&self, payload: String) -> Self {
        let mut builder = self.clone();
        builder.inner = builder.inner.payload(payload);
        builder
    }
}
