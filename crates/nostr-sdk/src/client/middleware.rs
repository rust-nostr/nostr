use std::sync::Arc;

use nostr::util::BoxedFuture;
use nostr::{Event, RelayUrl, SubscriptionId};
use nostr_relay_pool::policy::{AdmitPolicy, AdmitStatus, PolicyError};

use crate::gossip::Gossip;

#[derive(Debug)]
pub(crate) struct AdmissionPolicyMiddleware {
    pub(crate) gossip: Option<Gossip>,
    pub(crate) external_policy: Option<Arc<dyn AdmitPolicy>>,
}

impl AdmitPolicy for AdmissionPolicyMiddleware {
    fn admit_connection<'a>(
        &'a self,
        relay_url: &'a RelayUrl,
    ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
        Box::pin(async move {
            match &self.external_policy {
                Some(policy) => policy.admit_connection(relay_url).await,
                None => Ok(AdmitStatus::Success),
            }
        })
    }

    fn admit_event<'a>(
        &'a self,
        relay_url: &'a RelayUrl,
        subscription_id: &'a SubscriptionId,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<AdmitStatus, PolicyError>> {
        Box::pin(async move {
            // Process event in gossip
            if let Some(gossip) = &self.gossip {
                gossip.process_event(event).await;
            }

            // Check if event is allowed by external policy
            match &self.external_policy {
                Some(policy) => policy.admit_event(relay_url, subscription_id, event).await,
                None => Ok(AdmitStatus::Success),
            }
        })
    }
}
