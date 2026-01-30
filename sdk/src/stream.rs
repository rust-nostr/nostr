use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use tokio::sync::mpsc::Receiver;

/// Boxed stream
#[cfg(not(target_arch = "wasm32"))]
pub type BoxedStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;
/// Boxed stream
#[cfg(target_arch = "wasm32")]
pub type BoxedStream<T> = Pin<Box<dyn Stream<Item = T>>>;

#[derive(Debug)]
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
