// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use core::pin::Pin;
use core::task::{ready, Context, Poll};
use core::fmt;
use core::future::Future;
use core::mem::{size_of_val, align_of_val};

use async_utility::futures_util::Stream;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

/// A wrapper around [`mpsc::Receiver`] that implements [`Stream`].
#[derive(Debug)]
pub struct ReceiverStream<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> ReceiverStream<T> {
    /// Create a new `ReceiverStream`.
    #[inline]
    pub fn new(recv: mpsc::Receiver<T>) -> Self {
        Self { inner: recv }
    }

    /// Get back the inner [`mpsc::Receiver`].
    #[inline]
    pub fn into_inner(self) -> mpsc::Receiver<T> {
        self.inner
    }

    /// Closes the receiving half of a channel without dropping it.
    ///
    /// Check [`Receiver::close`] to learn more.
    #[inline]
    pub fn close(&mut self) {
        self.inner.close();
    }
}

impl<T> Stream for ReceiverStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_recv(cx)
    }
}

impl<T> AsRef<mpsc::Receiver<T>> for ReceiverStream<T> {
    fn as_ref(&self) -> &mpsc::Receiver<T> {
        &self.inner
    }
}

impl<T> AsMut<mpsc::Receiver<T>> for ReceiverStream<T> {
    fn as_mut(&mut self) -> &mut mpsc::Receiver<T> {
        &mut self.inner
    }
}


async fn make_future<T: Clone>(mut rx: broadcast::Receiver<T>) -> (Result<T, RecvError>, broadcast::Receiver<T>) {
    let result = rx.recv().await;
    (result, rx)
}

/// An error returned from the inner stream of a [`BroadcastStream`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BroadcastStreamRecvError {
    /// The receiver lagged too far behind. Attempting to receive again will
    /// return the oldest message still retained by the channel.
    ///
    /// Includes the number of skipped messages.
    Lagged(u64),
}

impl std::error::Error for BroadcastStreamRecvError {}

impl fmt::Display for BroadcastStreamRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BroadcastStreamRecvError::Lagged(amt) => write!(f, "channel lagged by {}", amt),
        }
    }
}

/// A wrapper around `tokio::broadcast::Receiver` that implements the `Stream` trait.
pub struct BroadcastStream<T> {
    inner: ReusableBoxFuture<'static, (Result<T, RecvError>, broadcast::Receiver<T>)>,
}

impl<T: Clone + Send + 'static> BroadcastStream<T> {
    /// Create a new `BroadcastStream`.
    pub fn new(rx: broadcast::Receiver<T>) -> Self {
        Self {
            inner: ReusableBoxFuture::new(make_future(rx)),
        }
    }
}

impl<T: Clone + Send + 'static> Stream for BroadcastStream<T> {
    type Item = Result<T, BroadcastStreamRecvError>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (result, rx) = ready!(self.inner.poll(cx));
        self.inner.set(make_future(rx));
        match result {
            Ok(item) => Poll::Ready(Some(Ok(item))),
            Err(RecvError::Closed) => Poll::Ready(None),
            Err(RecvError::Lagged(n)) => {
                Poll::Ready(Some(Err(BroadcastStreamRecvError::Lagged(n))))
            }
        }
    }
}

/// A reusable `Pin<Box<dyn Future<Output = T> + Send + 'a>>`.
///
/// This type lets you replace the future stored in the box without
/// reallocating when the size and alignment permit this.
struct ReusableBoxFuture<'a, T> {
    boxed: Pin<Box<dyn Future<Output = T> + Send + 'a>>,
}

impl<T> fmt::Debug for ReusableBoxFuture<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReusableBoxFuture").finish()
    }
}

impl<'a, T> ReusableBoxFuture<'a, T> {
    /// Create a new `ReusableBoxFuture<T>` containing the provided future.
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'a,
    {
        Self {
            boxed: Box::pin(future),
        }
    }

    /// Replace the future currently stored in this box.
    ///
    /// This reallocates if and only if the layout of the provided future is
    /// different from the layout of the currently stored future.
    pub fn set<F>(&mut self, future: F)
    where
        F: Future<Output = T> + Send + 'a,
    {
        if let Err(future) = self.try_set(future) {
            *self = Self::new(future);
        }
    }

    /// Replace the future currently stored in this box.
    ///
    /// This function never reallocates, but returns an error if the provided
    /// future has a different size or alignment from the currently stored
    /// future.
    pub fn try_set<F>(&mut self, future: F) -> Result<(), F>
    where
        F: Future<Output = T> + Send + 'a,
    {
        if size_of_val(&*self.boxed) == size_of_val(&future)
            && align_of_val(&*self.boxed) == align_of_val(&future)
        {
            // Replace the boxed future without deallocating.
            self.boxed = Box::pin(future);
            Ok(())
        } else {
            Err(future)
        }
    }

    /// Get a pinned reference to the underlying future.
    pub fn get_pin(&mut self) -> Pin<&mut (dyn Future<Output = T> + Send)> {
        self.boxed.as_mut()
    }

    /// Poll the future stored inside this box.
    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<T> {
        self.get_pin().poll(cx)
    }
}

impl<T> Future for ReusableBoxFuture<'_, T> {
    type Output = T;

    /// Poll the future stored inside this box.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        Pin::into_inner(self).get_pin().poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use async_utility::futures_util::{FutureExt, StreamExt};

    use super::*;

    #[test]
    fn test_different_futures() {
        let fut = async move { 10 };
        // Not zero sized!
        assert_eq!(Layout::for_value(&fut).size(), 1);

        let mut b = ReusableBoxFuture::new(fut);

        assert_eq!(b.get_pin().now_or_never(), Some(10));

        b.try_set(async move { 20 })
            .unwrap_or_else(|_| panic!("incorrect size"));

        assert_eq!(b.get_pin().now_or_never(), Some(20));

        b.try_set(async move { 30 })
            .unwrap_or_else(|_| panic!("incorrect size"));

        assert_eq!(b.get_pin().now_or_never(), Some(30));
    }

    #[test]
    fn test_different_sizes() {
        let fut1 = async move { 10 };
        let val = [0u32; 1000];
        let fut2 = async move { val[0] };
        let fut3 = ZeroSizedFuture {};

        assert_eq!(Layout::for_value(&fut1).size(), 1);
        assert_eq!(Layout::for_value(&fut2).size(), 4004);
        assert_eq!(Layout::for_value(&fut3).size(), 0);

        let mut b = ReusableBoxFuture::new(fut1);
        assert_eq!(b.get_pin().now_or_never(), Some(10));
        b.set(fut2);
        assert_eq!(b.get_pin().now_or_never(), Some(0));
        b.set(fut3);
        assert_eq!(b.get_pin().now_or_never(), Some(5));
    }

    struct ZeroSizedFuture {}
    impl Future for ZeroSizedFuture {
        type Output = u32;
        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<u32> {
            Poll::Ready(5)
        }
    }

    #[test]
    fn test_zero_sized() {
        let fut = ZeroSizedFuture {};
        // Zero sized!
        assert_eq!(Layout::for_value(&fut).size(), 0);

        let mut b = ReusableBoxFuture::new(fut);

        assert_eq!(b.get_pin().now_or_never(), Some(5));
        assert_eq!(b.get_pin().now_or_never(), Some(5));

        b.try_set(ZeroSizedFuture {})
            .unwrap_or_else(|_| panic!("incorrect size"));

        assert_eq!(b.get_pin().now_or_never(), Some(5));
        assert_eq!(b.get_pin().now_or_never(), Some(5));
    }

    #[tokio::test]
    async fn test_broadcast_stream_receives_messages() {
        let (tx, rx) = broadcast::channel(5); // Create a broadcast channel with a buffer size of 5
        let mut stream = BroadcastStream::new(rx);

        // Spawn a task to send messages
        tokio::spawn(async move {
            tx.send(1).unwrap();
            tx.send(2).unwrap();
            tx.send(3).unwrap();
        });

        // Collect messages from the stream
        let mut received = Vec::new();
        while let Some(Ok(msg)) = stream.next().await {
            received.push(msg);
            if received.len() == 3 {
                break;
            }
        }

        assert_eq!(received, vec![1, 2, 3]); // Ensure the stream received the correct messages
    }

    #[tokio::test]
    async fn test_broadcast_stream_handles_channel_closure() {
        let (tx, rx) = broadcast::channel::<()>(5); // Create a broadcast channel
        let mut stream = BroadcastStream::new(rx);

        // Drop the transmitter to close the channel
        drop(tx);

        let result = stream.next().await; // The stream ends when the channel closes
        assert_eq!(result, None); // Ensure stream returns `None` on channel closure
    }

    #[tokio::test]
    async fn test_broadcast_stream_lagged_error() {
        let (tx, rx) = broadcast::channel(2); // Create a broadcast channel with a small buffer size
        let mut stream = BroadcastStream::new(rx);

        // Fill the buffer until it lags
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap(); // The receiver will lag here since it can't keep up

        let first = stream.next().await.unwrap(); // First item will be an error
        assert!(matches!(first, Err(BroadcastStreamRecvError::Lagged(_))));
    }

    #[tokio::test]
    async fn test_broadcast_stream_multiple_receivers() {
        let (tx, rx1) = broadcast::channel(5);
        let rx2 = tx.subscribe(); // Create a second receiver

        let mut stream1 = BroadcastStream::new(rx1);
        let mut stream2 = BroadcastStream::new(rx2);

        // Spawn a task to send messages
        tokio::spawn(async move {
            tx.send(42).unwrap();
        });

        // Both streams should receive the same message
        let result1 = stream1.next().await.unwrap();
        let result2 = stream2.next().await.unwrap();

        assert_eq!(result1, Ok(42));
        assert_eq!(result2, Ok(42));
    }

    #[tokio::test]
    async fn test_broadcast_stream_empty_channel() {
        let (_tx, rx) = broadcast::channel::<i32>(5); // Create a broadcast channel
        let mut stream = BroadcastStream::new(rx);

        // Drop the transmitter (`_tx`) to close the channel
        drop(_tx);

        let result = stream.next().await; // No messages to receive
        assert_eq!(result, None); // Stream should terminate cleanly
    }
}
