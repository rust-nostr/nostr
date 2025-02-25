// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::sync::Arc;

use nostr_sdk::pool::policy;
use uniffi::Enum;

use crate::error::Result;
use crate::protocol::event::Event;

#[derive(Enum)]
pub enum AdmitStatus {
    Success,
    Rejected,
}

impl From<AdmitStatus> for policy::AdmitStatus {
    fn from(status: AdmitStatus) -> Self {
        match status {
            AdmitStatus::Success => Self::Success,
            AdmitStatus::Rejected => Self::Rejected,
        }
    }
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait AdmitPolicy: Send + Sync {
    /// Admit Event
    ///
    /// Returns `AdmitStatus::Success` if the event is admitted, otherwise `AdmitStatus::Rejected`.
    async fn admit_event(
        &self,
        relay_url: String,
        subscription_id: String,
        event: Arc<Event>,
    ) -> Result<AdmitStatus>;
}

pub(crate) struct FFI2RustAdmitPolicy {
    pub(crate) inner: Arc<dyn AdmitPolicy>,
}

impl fmt::Debug for FFI2RustAdmitPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FFI2RustAdmitPolicy").finish()
    }
}

mod inner {
    use std::sync::Arc;

    use nostr::prelude::BoxedFuture;
    use nostr::{Event, RelayUrl, SubscriptionId};
    use nostr_sdk::pool::policy::AdmitPolicy;
    use nostr_sdk::prelude::{AdmitStatus, PolicyError};

    use super::FFI2RustAdmitPolicy;
    use crate::error::MiddleError;

    impl AdmitPolicy for FFI2RustAdmitPolicy {
        fn admit_event<'a>(
            &'a self,
            relay_url: &'a RelayUrl,
            subscription_id: &'a SubscriptionId,
            event: &'a Event,
        ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
            Box::pin(async move {
                let event = Arc::new(event.clone().into());
                self.inner
                    .admit_event(relay_url.to_string(), subscription_id.to_string(), event)
                    .await
                    .map(|s| s.into())
                    .map_err(MiddleError::from)
                    .map_err(PolicyError::backend)
            })
        }
    }
}
