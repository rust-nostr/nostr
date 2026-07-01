use std::collections::HashMap;
use std::future::IntoFuture;

use nostr::{Filter, RelayUrl, SubscriptionId};

use super::output::Output;
use super::req_target::ReqTarget;
use super::util::build_targets;
use crate::client::{Client, Error};
use crate::future::BoxedFuture;
use crate::relay::SubscribeAutoCloseOptions;

/// Subscribe to events
#[must_use = "Does nothing unless you await!"]
pub struct Subscribe<'client, 'url> {
    client: &'client Client,
    target: ReqTarget<'url>,
    id: Option<SubscriptionId>,
    auto_close: Option<SubscribeAutoCloseOptions>,
}

impl<'client, 'url> Subscribe<'client, 'url> {
    #[inline]
    pub(crate) fn new(client: &'client Client, target: ReqTarget<'url>) -> Self {
        Self {
            client,
            target,
            id: None,
            auto_close: None,
        }
    }

    /// Set a specific subscription ID
    #[inline]
    pub fn with_id(mut self, id: SubscriptionId) -> Self {
        self.id = Some(id);
        self
    }

    /// Set auto-close conditions
    #[inline]
    pub fn close_on(mut self, opts: SubscribeAutoCloseOptions) -> Self {
        self.auto_close = Some(opts);
        self
    }
}

impl<'client, 'url> IntoFuture for Subscribe<'client, 'url>
where
    'url: 'client,
{
    type Output = Result<Output<SubscriptionId>, Error>;
    type IntoFuture = BoxedFuture<'client, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Build targets
            let targets: HashMap<RelayUrl, Vec<Filter>> =
                build_targets(self.client, self.target).await?;

            self.client
                .pool()
                .subscribe(targets, self.id, self.auto_close)
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::StreamExt;
    use nostr::event::{EventBuilder, FinalizeEvent};
    use nostr::{Filter, Keys, Kind, SubscriptionId};

    use crate::authenticator::SignerAuthenticator;
    use crate::client::ClientNotification;
    use crate::test_utils::{
        setup_client, setup_client_with_authenticator, setup_nip42_read_local_relay,
    };

    #[tokio::test]
    async fn test_client_subscribe_dont_resubscribes_after_auth_required_closed_without_authenticator()
     {
        let local = setup_nip42_read_local_relay().await;

        let keys = Keys::generate();
        let event = EventBuilder::text_note("Test").finalize(&keys).unwrap();
        local.add_event(event).await.unwrap();

        let client = setup_client(local.url().await).await;
        let id = SubscriptionId::new("client-auth-required-subscribe");

        let output = client
            .subscribe(Filter::new().kind(Kind::TextNote).limit(1))
            .with_id(id.clone())
            .await
            .unwrap();
        assert!(output.failed.is_empty());

        tokio::time::sleep(Duration::from_secs(1)).await;

        assert!(!client.subscriptions().await.contains_key(&id));
    }

    #[tokio::test]
    async fn test_client_subscribe_resubscribes_after_auth_required_closed() {
        let local = setup_nip42_read_local_relay().await;

        let keys = Keys::generate();
        let expected = EventBuilder::text_note("Test").finalize(&keys).unwrap();
        local.add_event(expected.clone()).await.unwrap();

        let authenticator = SignerAuthenticator::new(keys);
        let client = setup_client_with_authenticator(local.url().await, authenticator).await;

        let filter = Filter::new().kind(Kind::TextNote).limit(1);

        let id = SubscriptionId::new("client-auth-required-subscribe");

        let mut notifications = client.notifications();

        let output = client.subscribe(filter).with_id(id.clone()).await.unwrap();
        assert!(output.failed.is_empty());

        let received = tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(notification) = notifications.next().await {
                if let ClientNotification::Event {
                    subscription_id,
                    event,
                    ..
                } = notification
                {
                    if subscription_id == id {
                        return event;
                    }
                }
            }

            panic!("notifications ended before event was received");
        })
        .await
        .unwrap();

        assert_eq!(received.id, expected.id);
    }
}
