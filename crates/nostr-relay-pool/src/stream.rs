// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Streams

use std::pin::Pin;
use std::task::{Context, Poll};

use async_utility::futures_util::Stream;
use tokio::sync::mpsc::Receiver;

/// A wrapper around [`Receiver`] that implements [`Stream`].
#[derive(Debug)]
pub struct ReceiverStream<T> {
    inner: Receiver<T>,
}

impl<T> ReceiverStream<T> {
    /// Create a new `ReceiverStream`.
    #[inline]
    pub fn new(recv: Receiver<T>) -> Self {
        Self { inner: recv }
    }

    /// Get back the inner [`Receiver`].
    #[inline]
    pub fn into_inner(self) -> Receiver<T> {
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

impl<T> AsRef<Receiver<T>> for ReceiverStream<T> {
    fn as_ref(&self) -> &Receiver<T> {
        &self.inner
    }
}

impl<T> AsMut<Receiver<T>> for ReceiverStream<T> {
    fn as_mut(&mut self) -> &mut Receiver<T> {
        &mut self.inner
    }
}
