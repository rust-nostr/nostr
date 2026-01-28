use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::Stream;
use nostr::{Event, Filter, SubscriptionId};
use tokio::sync::mpsc;

use super::subscribe::subscribe_auto_closing;
use crate::relay::{
    Error, Relay, ReqExitPolicy, SubscribeAutoCloseOptions, SubscriptionActivity,
    SubscriptionAutoClosedReason,
};
use crate::stream::BoxedStream;

type EventStream = BoxedStream<Result<Event, Error>>;

/// Stream events
#[must_use = "Does nothing unless you await!"]
pub struct StreamEvents<'relay> {
    relay: &'relay Relay,
    filters: Vec<Filter>,
    timeout: Option<Duration>,
    policy: ReqExitPolicy,
}

impl<'relay> StreamEvents<'relay> {
    pub(crate) fn new(relay: &'relay Relay, filters: Vec<Filter>) -> Self {
        Self {
            relay,
            filters,
            timeout: None,
            policy: ReqExitPolicy::ExitOnEOSE,
        }
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

    async fn exec(self) -> Result<EventStream, Error> {
        // Create channels
        let (tx, rx) = mpsc::channel(512);

        // Compose auto-closing options
        let opts: SubscribeAutoCloseOptions = SubscribeAutoCloseOptions::default()
            .exit_policy(self.policy)
            .timeout(self.timeout);

        // Subscribe
        let id: SubscriptionId = SubscriptionId::generate();
        subscribe_auto_closing(self.relay, id, self.filters, opts, Some(tx)).await?;

        Ok(Box::pin(SubscriptionActivityEventStream::new(rx)))
    }
}

impl<'relay> IntoFuture for StreamEvents<'relay> {
    type Output = Result<EventStream, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'relay>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}

struct SubscriptionActivityEventStream {
    rx: mpsc::Receiver<SubscriptionActivity>,
    done: bool,
}

impl SubscriptionActivityEventStream {
    fn new(rx: mpsc::Receiver<SubscriptionActivity>) -> Self {
        Self { rx, done: false }
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
