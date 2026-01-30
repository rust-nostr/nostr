use std::borrow::Cow;
use std::future::{Future, IntoFuture};
use std::pin::Pin;

use nostr::{ClientMessage, Filter, SubscriptionId};
use tokio::sync::mpsc;

use crate::relay::{Error, Relay, SubscribeAutoCloseOptions, SubscriptionActivity};

/// Subscribe to events
#[must_use = "Does nothing unless you await!"]
pub struct Subscribe<'relay> {
    relay: &'relay Relay,
    filters: Vec<Filter>,
    id: Option<SubscriptionId>,
    auto_close: Option<SubscribeAutoCloseOptions>,
}

impl<'relay> Subscribe<'relay> {
    #[inline]
    pub(crate) fn new(relay: &'relay Relay, filters: Vec<Filter>) -> Self {
        Self {
            relay,
            filters,
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

pub(super) async fn subscribe_auto_closing(
    relay: &Relay,
    id: SubscriptionId,
    filters: Vec<Filter>,
    opts: SubscribeAutoCloseOptions,
    activity: Option<mpsc::Sender<SubscriptionActivity>>,
) -> Result<(), Error> {
    // Check if filters are empty
    if filters.is_empty() {
        return Err(Error::EmptyFilters);
    }

    // Compose REQ message
    let msg: ClientMessage = ClientMessage::Req {
        subscription_id: Cow::Borrowed(&id),
        filters: filters.iter().map(Cow::Borrowed).collect(),
    };

    // Subscribe to notifications
    let notifications = relay.inner.internal_notification_sender.subscribe();

    // Register the auto-closing subscription
    relay
        .inner
        .add_auto_closing_subscription(id.clone(), filters.clone())
        .await;

    // Send REQ message
    if let Err(e) = relay.send_msg(msg).await {
        // Remove previously added subscription
        relay.inner.remove_subscription(&id).await;

        // Propagate error
        return Err(e);
    }

    // Spawn auto-closing handler
    relay
        .inner
        .spawn_auto_closing_handler(id, filters, opts, notifications, activity);

    // Return
    Ok(())
}

async fn subscribe_long_lived(
    relay: &Relay,
    id: SubscriptionId,
    filters: Vec<Filter>,
) -> Result<(), Error> {
    // Check if filters are empty
    if filters.is_empty() {
        return Err(Error::EmptyFilters);
    }

    // Compose REQ message
    let msg: ClientMessage = ClientMessage::Req {
        subscription_id: Cow::Borrowed(&id),
        filters: filters.iter().map(Cow::Borrowed).collect(),
    };

    // Send REQ message
    relay.send_msg(msg).await?;

    // No auto-close subscription: update subscription filter
    relay.inner.update_subscription(id, filters, true).await;

    // Return
    Ok(())
}

impl<'relay> IntoFuture for Subscribe<'relay> {
    type Output = Result<SubscriptionId, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'relay>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Get or generate subscription ID
            let id: SubscriptionId = self.id.unwrap_or_else(SubscriptionId::generate);

            // Check if the auto-close condition is set
            match self.auto_close {
                Some(opts) => {
                    subscribe_auto_closing(self.relay, id.clone(), self.filters, opts, None).await?
                }
                None => subscribe_long_lived(self.relay, id.clone(), self.filters).await?,
            }

            // Return subscription ID
            Ok(id)
        })
    }
}

impl_blocking!(Subscribe<'_>);

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_utility::time;
    use nostr::{Event, EventBuilder, EventId, Keys, Kind};
    use nostr_relay_builder::prelude::*;

    use super::*;
    use crate::prelude::RelayNotification;
    use crate::relay::{RelayOptions, RelayStatus};

    #[tokio::test]
    async fn test_subscribe_ban_relay() {
        // Mock relay
        let opts = LocalRelayTestOptions {
            unresponsive_connection: None,
            send_random_events: true,
        };
        let mock = MockRelay::run_with_opts(opts).await.unwrap();
        let url = mock.url().await;

        let relay = Relay::builder(url)
            .opts(
                RelayOptions::default()
                    .verify_subscriptions(true)
                    .ban_relay_on_mismatch(true),
            )
            .build();

        assert_eq!(relay.status(), RelayStatus::Initialized);

        relay
            .try_connect()
            .timeout(Duration::from_secs(3))
            .await
            .unwrap();

        assert_eq!(relay.status(), RelayStatus::Connected);

        let filter = Filter::new().kind(Kind::Metadata).limit(3);
        relay.subscribe(filter).await.unwrap();

        // Keep up the test
        time::timeout(
            Some(Duration::from_secs(10)),
            relay.handle_notifications(|_| async { Ok(false) }),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(relay.status(), RelayStatus::Banned);

        assert!(!relay.inner.is_running());
    }

    #[tokio::test]
    async fn test_subscribe_ephemeral_event() {
        // Mock relay
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        // Sender
        let relay1: Relay = Relay::new(url.clone());
        relay1.connect();
        relay1
            .try_connect()
            .timeout(Duration::from_millis(500))
            .await
            .unwrap();

        // Fetcher
        let relay2 = Relay::new(url.clone());
        relay2
            .try_connect()
            .timeout(Duration::from_millis(500))
            .await
            .unwrap();

        // Signer
        let keys = Keys::generate();

        // Event
        let kind = Kind::Custom(22_222); // Ephemeral kind
        let event: Event = EventBuilder::new(kind, "").sign_with_keys(&keys).unwrap();

        let event_id: EventId = event.id;

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            relay1.send_event(&event).await.unwrap();
        });

        // Subscribe
        let filter = Filter::new().kind(kind);
        let sub_id = relay2.subscribe(filter).await.unwrap();

        // Listen for notifications
        let fut = relay2.handle_notifications(|notification| async {
            if let RelayNotification::Event {
                subscription_id,
                event,
            } = notification
            {
                if subscription_id == sub_id {
                    if event.id == event_id {
                        return Ok(true);
                    } else {
                        panic!("Unexpected event");
                    }
                } else {
                    panic!("Unexpected subscription ID");
                }
            }
            Ok(false)
        });

        tokio::time::timeout(Duration::from_secs(5), fut)
            .await
            .unwrap()
            .unwrap();
    }
}
