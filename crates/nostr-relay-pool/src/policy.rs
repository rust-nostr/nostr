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
            Self::Backend(e) => e.fmt(f),
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdmitStatus {
    /// Admission succeeds
    Success,
    /// Admission rejected
    Rejected {
        /// Optional reason
        reason: Option<String>,
    },
}

impl AdmitStatus {
    /// Success
    #[inline]
    pub fn success() -> Self {
        Self::Success
    }

    /// Rejection with reason
    #[inline]
    pub fn rejected<S>(reason: S) -> Self
    where
        S: Into<String>,
    {
        Self::Rejected {
            reason: Some(reason.into()),
        }
    }
}

/// Admission policy
pub trait AdmitPolicy: fmt::Debug + Send + Sync {
    /// Admit connecting to a relay
    ///
    /// Returns [`AdmitStatus::Success`] if the connection is allowed, otherwise [`AdmitStatus::Rejected`].
    fn admit_connection<'a>(
        &'a self,
        relay_url: &'a RelayUrl,
    ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
        let _ = relay_url;
        Box::pin(async move { Ok(AdmitStatus::Success) })
    }

    /// Admit [`Event`]
    ///
    /// Returns [`AdmitStatus::Success`] if the event is admitted, otherwise [`AdmitStatus::Rejected`].
    fn admit_event<'a>(
        &'a self,
        relay_url: &'a RelayUrl,
        subscription_id: &'a SubscriptionId,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
        let _ = (relay_url, subscription_id, event);
        Box::pin(async move { Ok(AdmitStatus::Success) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admit_status_success() {
        let status = AdmitStatus::success();
        assert_eq!(status, AdmitStatus::Success);
    }

    #[test]
    fn test_admit_status_rejcted() {
        let status = AdmitStatus::rejected("not admitted");
        assert_eq!(
            status,
            AdmitStatus::Rejected {
                reason: Some(String::from("not admitted"))
            }
        );
    }
}
