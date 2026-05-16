use std::future::IntoFuture;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::Stream;
use nostr::{Event, Filter, SubscriptionId};
use tokio::sync::{mpsc, oneshot};

use super::subscribe::subscribe_auto_closing;
use crate::future::BoxedFuture;
use crate::relay::{
    Error, Relay, ReqExitPolicy, SubscribeAutoCloseOptions, SubscriptionActivity,
    SubscriptionAutoClosedReason,
};

type EventStream = Pin<Box<dyn Stream<Item = Result<Event, Error>> + Send>>;

/// Stream events
#[must_use = "Does nothing unless you await!"]
pub struct StreamEvents<'relay> {
    relay: &'relay Relay,
    filters: Vec<Filter>,
    id: Option<SubscriptionId>,
    timeout: Option<Duration>,
    policy: ReqExitPolicy,
}

impl<'relay> StreamEvents<'relay> {
    pub(crate) fn new(relay: &'relay Relay, filters: Vec<Filter>) -> Self {
        Self {
            relay,
            filters,
            id: None,
            timeout: None,
            policy: ReqExitPolicy::ExitOnEOSE,
        }
    }

    /// Set a specific subscription ID
    #[inline]
    pub fn with_id(mut self, id: SubscriptionId) -> Self {
        self.id = Some(id);
        self
    }

    #[inline]
    pub(crate) fn maybe_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set a timeout
    ///
    /// By default, no timeout is configured.
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set request exit policy (default: [`ReqExitPolicy::ExitOnEOSE`]).
    #[inline]
    pub fn policy(mut self, policy: ReqExitPolicy) -> Self {
        self.policy = policy;
        self
    }
}

impl<'relay> IntoFuture for StreamEvents<'relay> {
    type Output = Result<EventStream, Error>;
    type IntoFuture = BoxedFuture<'relay, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            // Create channels
            let (tx, rx) = mpsc::channel(512);

            // Compose auto-closing options
            let opts: SubscribeAutoCloseOptions = SubscribeAutoCloseOptions::default()
                .exit_policy(self.policy)
                .timeout(self.timeout);

            // Get or generate a subscription ID
            let id: SubscriptionId = self.id.unwrap_or_else(SubscriptionId::generate);

            // Subscribe
            let (cancel_tx, cancel_rx) = oneshot::channel();
            subscribe_auto_closing(
                self.relay,
                id,
                self.filters,
                opts,
                Some(tx),
                Some(cancel_rx),
            )
            .await?;

            Ok(Box::pin(SubscriptionActivityEventStream::new(rx, cancel_tx)) as EventStream)
        })
    }
}

struct SubscriptionActivityEventStream {
    rx: mpsc::Receiver<SubscriptionActivity>,
    done: bool,
    cancel: Option<oneshot::Sender<()>>,
}

impl SubscriptionActivityEventStream {
    fn new(rx: mpsc::Receiver<SubscriptionActivity>, cancel: oneshot::Sender<()>) -> Self {
        Self {
            rx,
            done: false,
            cancel: Some(cancel),
        }
    }
}

impl Drop for SubscriptionActivityEventStream {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            let _ = cancel.send(());
        }
    }
}

impl Stream for SubscriptionActivityEventStream {
    type Item = Result<Event, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.rx).poll_recv(cx) {
            Poll::Ready(Some(activity)) => match activity {
                SubscriptionActivity::ReceivedEvent(event) => Poll::Ready(Some(Ok(event))),
                SubscriptionActivity::Closed(reason) => match reason {
                    SubscriptionAutoClosedReason::AuthenticationFailed => {
                        self.done = true;
                        Poll::Ready(Some(Err(Error::AuthenticationFailed)))
                    }
                    SubscriptionAutoClosedReason::Closed(message) => {
                        self.done = true;
                        Poll::Ready(Some(Err(Error::RelayMessage(message))))
                    }
                    SubscriptionAutoClosedReason::Completed => {
                        self.done = true;
                        Poll::Ready(None)
                    }
                },
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::StreamExt;
    use nostr::event::EventBuilder;
    use nostr::key::Keys;
    use nostr::{Filter, Kind, SubscriptionId};
    use nostr_relay_builder::MockRelay;

    use super::*;
    use crate::relay::{Relay, RelayOptions};

    #[tokio::test]
    async fn test_stream_terminates_on_drop() {
        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        let relay = Relay::new(url);

        relay
            .try_connect()
            .timeout(Duration::from_secs(3))
            .await
            .unwrap();

        let filter = Filter::new().kind(Kind::TextNote).limit(1);
        let id = SubscriptionId::generate();

        let stream = relay
            .stream_events(filter)
            .with_id(id.clone())
            .policy(ReqExitPolicy::WaitForEvents(1))
            .await
            .unwrap();

        // Check if relay has the stream subscription
        let exists: bool = relay.subscription(&id).await.is_some();
        assert!(exists);

        // Drop the stream
        // This must terminate the stream and close the subscription
        drop(stream);

        // Wait a bit
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Now the subscription must not exist anymore
        let exists: bool = relay.subscription(&id).await.is_some();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_stream_with_subscription_verification_single_filter() {
        let keys = Keys::generate();
        let event = EventBuilder::text_note("test").sign(&keys).unwrap();

        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        mock.add_event(event.clone()).await.unwrap();

        let opts = RelayOptions::default()
            .verify_subscriptions(true)
            .ban_relay_on_mismatch(true);
        let relay = Relay::builder(url).opts(opts).build();

        relay.connect();

        let filter = Filter::new().author(event.pubkey).kind(Kind::TextNote);

        let mut stream = relay
            .stream_events(filter)
            .timeout(Duration::from_secs(3))
            .await
            .unwrap();

        let streamed_event = stream
            .next()
            .await
            .expect("Received None instead of the event")
            .unwrap();
        assert_eq!(streamed_event.id, event.id);
    }

    #[tokio::test]
    async fn test_stream_with_subscription_verification_multiple_filters() {
        let keys = Keys::generate();
        let event = EventBuilder::text_note("test").sign(&keys).unwrap();

        let mock = MockRelay::run().await.unwrap();
        let url = mock.url().await;

        mock.add_event(event.clone()).await.unwrap();

        let opts = RelayOptions::default()
            .verify_subscriptions(true)
            .ban_relay_on_mismatch(true);
        let relay = Relay::builder(url).opts(opts).build();

        relay.connect();

        let matching_filter = Filter::new().author(event.pubkey).kind(Kind::TextNote);
        let non_matching_filter = Filter::new().author(event.pubkey).kind(Kind::Repost);

        let mut stream = relay
            .stream_events([matching_filter, non_matching_filter])
            .timeout(Duration::from_secs(3))
            .await
            .unwrap();

        let streamed_event = stream
            .next()
            .await
            .expect("Received None instead of the event")
            .unwrap();
        assert_eq!(streamed_event.id, event.id);
    }
}
