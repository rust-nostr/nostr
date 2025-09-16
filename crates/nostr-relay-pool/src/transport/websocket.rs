// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! WebSocket transport

use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use async_utility::futures_util::stream::SplitSink;
use async_wsocket::futures_util::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use async_wsocket::{ConnectionMode, Message, WebSocket};
use nostr::Url;
use nostr::util::BoxedFuture;

use super::error::TransportError;

/// WebSocket transport sink
#[cfg(not(target_arch = "wasm32"))]
pub type BoxSink = Box<dyn Sink<Message, Error = TransportError> + Send + Unpin>;
/// WebSocket transport stream
#[cfg(not(target_arch = "wasm32"))]
pub type BoxStream = Box<dyn Stream<Item = Result<Message, TransportError>> + Send + Unpin>;
/// WebSocket transport sink
#[cfg(target_arch = "wasm32")]
pub type BoxSink = Box<dyn Sink<Message, Error = TransportError> + Unpin>;
/// WebSocket transport stream
#[cfg(target_arch = "wasm32")]
pub type BoxStream = Box<dyn Stream<Item = Result<Message, TransportError>> + Unpin>;

#[doc(hidden)]
pub trait IntoWebSocketTransport {
    fn into_transport(self) -> Arc<dyn WebSocketTransport>;
}

impl IntoWebSocketTransport for Arc<dyn WebSocketTransport> {
    fn into_transport(self) -> Arc<dyn WebSocketTransport> {
        self
    }
}

impl<T> IntoWebSocketTransport for T
where
    T: WebSocketTransport + Sized + 'static,
{
    fn into_transport(self) -> Arc<dyn WebSocketTransport> {
        Arc::new(self)
    }
}

impl<T> IntoWebSocketTransport for Arc<T>
where
    T: WebSocketTransport + 'static,
{
    fn into_transport(self) -> Arc<dyn WebSocketTransport> {
        self
    }
}

/// WebSocket transport
pub trait WebSocketTransport: fmt::Debug + Send + Sync {
    /// Support ping/pong
    fn support_ping(&self) -> bool;

    /// Connect
    fn connect<'a>(
        &'a self,
        url: &'a Url,
        mode: &'a ConnectionMode,
        timeout: Duration,
    ) -> BoxedFuture<'a, Result<(BoxSink, BoxStream), TransportError>>;
}

/// Default websocket transport
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefaultWebsocketTransport;

impl WebSocketTransport for DefaultWebsocketTransport {
    fn support_ping(&self) -> bool {
        true
    }

    fn connect<'a>(
        &'a self,
        url: &'a Url,
        mode: &'a ConnectionMode,
        timeout: Duration,
    ) -> BoxedFuture<'a, Result<(BoxSink, BoxStream), TransportError>> {
        Box::pin(async move {
            // Connect
            let socket: WebSocket = WebSocket::connect(url, mode, timeout)
                .await
                .map_err(TransportError::backend)?;

            // Split sink and stream
            let (tx, rx) = socket.split();

            // NOTE: don't use sink_map_err here, as it may cause panics!
            // Issue: https://github.com/rust-nostr/nostr/issues/984
            let sink: BoxSink = Box::new(TransportSink(tx)) as BoxSink;
            let stream: BoxStream = Box::new(rx.map_err(TransportError::backend)) as BoxStream;

            Ok((sink, stream))
        })
    }
}

struct TransportSink(SplitSink<WebSocket, Message>);

impl Sink<Message> for TransportSink {
    type Error = TransportError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_ready_unpin(cx)
            .map_err(TransportError::backend)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        Pin::new(&mut self.0)
            .start_send_unpin(item)
            .map_err(TransportError::backend)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_flush_unpin(cx)
            .map_err(TransportError::backend)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_close_unpin(cx)
            .map_err(TransportError::backend)
    }
}
