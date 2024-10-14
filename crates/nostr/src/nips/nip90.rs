// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP90: Data Vending Machines
//!
//! <https://github.com/nostr-protocol/nips/blob/master/90.md>

use alloc::string::String;
use core::fmt;
use core::str::FromStr;

use crate::{Event, EventId, PublicKey};

/// DVM Error
#[derive(Debug)]
pub enum Error {
    /// Unknown status
    UnknownStatus,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownStatus => write!(f, "Unknown status"),
        }
    }
}

/// Data Vending Machine Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataVendingMachineStatus {
    /// Service Provider requires payment before continuing
    PaymentRequired,
    /// Service Provider is processing the job
    Processing,
    /// Service Provider was unable to process the job
    Error,
    /// Service Provider successfully processed the job
    Success,
    /// Service Provider partially processed the job
    Partial,
}

impl fmt::Display for DataVendingMachineStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PaymentRequired => write!(f, "payment-required"),
            Self::Processing => write!(f, "processing"),
            Self::Error => write!(f, "error"),
            Self::Success => write!(f, "success"),
            Self::Partial => write!(f, "partial"),
        }
    }
}

impl FromStr for DataVendingMachineStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "payment-required" => Ok(Self::PaymentRequired),
            "processing" => Ok(Self::Processing),
            "error" => Ok(Self::Error),
            "success" => Ok(Self::Success),
            "partial" => Ok(Self::Partial),
            _ => Err(Error::UnknownStatus),
        }
    }
}

/// Data Vending Machine (DVM) - Job Feedback data
///
/// <https://github.com/nostr-protocol/nips/blob/master/90.md>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobFeedbackData {
    pub(crate) job_request_id: EventId,
    pub(crate) customer_public_key: PublicKey,
    pub(crate) status: DataVendingMachineStatus,
    pub(crate) extra_info: Option<String>,
    pub(crate) amount_msat: Option<u64>,
    pub(crate) bolt11: Option<String>,
    pub(crate) payload: Option<String>,
}

impl JobFeedbackData {
    /// Construct new Job Feedback
    pub fn new(job_request: &Event, status: DataVendingMachineStatus) -> Self {
        Self {
            job_request_id: job_request.id,
            customer_public_key: job_request.pubkey,
            status,
            extra_info: None,
            amount_msat: None,
            bolt11: None,
            payload: None,
        }
    }

    /// Add extra info
    #[inline]
    pub fn extra_info<S>(mut self, info: S) -> Self
    where
        S: Into<String>,
    {
        self.extra_info = Some(info.into());
        self
    }

    /// Add payment amount
    #[inline]
    pub fn amount(mut self, millisats: u64, bolt11: Option<String>) -> Self {
        self.amount_msat = Some(millisats);
        self.bolt11 = bolt11;
        self
    }

    /// Add payload
    #[inline]
    pub fn payload<S>(mut self, payload: S) -> Self
    where
        S: Into<String>,
    {
        self.payload = Some(payload.into());
        self
    }
}
