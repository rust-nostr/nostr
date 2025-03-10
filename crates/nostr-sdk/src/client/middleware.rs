// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::borrow::Cow;
use std::sync::Arc;

use nostr::util::BoxedFuture;
use nostr::{Event, RelayUrl, SubscriptionId};
use nostr_gossip::Gossip;
use nostr_relay_pool::policy::{AdmitPolicy, AdmitStatus, PolicyError};

#[derive(Debug)]
pub(super) struct Middleware {
    pub(super) gossip: Option<Gossip>,
    pub(super) external_policy: Option<Arc<dyn AdmitPolicy>>,
}

impl AdmitPolicy for Middleware {
    fn admit_event<'a>(
        &'a self,
        relay_url: &'a RelayUrl,
        subscription_id: &'a SubscriptionId,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
        Box::pin(async move {
            if let Some(gossip) = &self.gossip {
                gossip.process_event(Cow::Borrowed(event));
            }

            if let Some(external_policy) = &self.external_policy {
                return external_policy
                    .admit_event(relay_url, subscription_id, event)
                    .await;
            }

            Ok(AdmitStatus::Success)
        })
    }
}
