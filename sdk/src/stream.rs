use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Stream, StreamExt};
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::BroadcastStream;

/// Boxed stream
#[cfg(not(target_arch = "wasm32"))]
pub type BoxedStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;
/// Boxed stream
#[cfg(target_arch = "wasm32")]
pub type BoxedStream<T> = Pin<Box<dyn Stream<Item = T>>>;

pub(crate) struct ReceiverStream<T> {
    inner: Receiver<T>,
}

impl<T> ReceiverStream<T> {
    #[inline]
    pub(crate) fn new(recv: Receiver<T>) -> Self {
        Self { inner: recv }
    }
}

impl<T> Stream for ReceiverStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_recv(cx)
    }
}

pub(crate) struct NotificationStream<T> {
    inner: BroadcastStream<T>,
}

impl<T> NotificationStream<T>
where
    T: Clone + Send + 'static,
{
    #[inline]
    pub(crate) fn new(inner: broadcast::Receiver<T>) -> Self {
        Self {
            inner: BroadcastStream::new(inner),
        }
    }
}

impl<T> Stream for NotificationStream<T>
where
    T: Clone + Send + 'static,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.inner.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(notification))) => return Poll::Ready(Some(notification)),
                // Skip errors for now
                Poll::Ready(Some(Err(..))) => continue,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
