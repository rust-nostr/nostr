// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Policies

use std::fmt;

use nostr::util::BoxedFuture;
use nostr::{Event, RelayUrl, SubscriptionId};

/// Policy Error
#[derive(Debug)]
pub enum PolicyError {
    /// An error happened in the underlying backend.
    Backend(Box<dyn std::error::Error + Send + Sync>),
}

impl std::error::Error for PolicyError {}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(e) => write!(f, "{e}"),
        }
    }
}

impl PolicyError {
    /// Create a new backend error
    ///
    /// Shorthand for `Error::Backend(Box::new(error))`.
    #[inline]
    pub fn backend<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Backend(Box::new(error))
    }
}

/// Admission status
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdmitStatus {
    /// Admission succeeds
    Success,
    /// Admission rejected
    Rejected,
}

/// Admission policy
pub trait AdmitPolicy: fmt::Debug + Send + Sync {
    /// Admit [`Event`]
    ///
    /// Returns [`AdmitStatus::Success`] if the event is admitted, otherwise [`AdmitStatus::Rejected`].
    fn admit_event<'a>(
        &'a self,
        relay_url: &'a RelayUrl,
        subscription_id: &'a SubscriptionId,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>>;
}
