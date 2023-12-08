// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP90
//!
//! <https://github.com/nostr-protocol/nips/blob/master/90.md>

use core::fmt;
use core::str::FromStr;

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
