// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! WebSocket transport

use std::fmt;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_wsocket::{ConnectionMode, Message, WebSocket};
use futures::stream::SplitSink;
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use nostr::Url;

use crate::error::Error;
use crate::future::BoxedFuture;

/// WebSocket transport sink
#[cfg(not(target_arch = "wasm32"))]
pub type WebSocketSink = Pin<Box<dyn Sink<Message, Error = Error> + Send>>;
/// WebSocket transport sink
#[cfg(target_arch = "wasm32")]
pub type WebSocketSink = Pin<Box<dyn Sink<Message, Error = Error>>>;
/// WebSocket transport stream
#[cfg(not(target_arch = "wasm32"))]
pub type WebSocketStream = Pin<Box<dyn Stream<Item = Result<Message, Error>> + Send>>;
/// WebSocket transport stream
#[cfg(target_arch = "wasm32")]
pub type WebSocketStream = Pin<Box<dyn Stream<Item = Result<Message, Error>>>>;

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
        proxy: Option<SocketAddr>,
    ) -> BoxedFuture<'a, Result<(WebSocketSink, WebSocketStream), Error>>;
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
        proxy: Option<SocketAddr>,
    ) -> BoxedFuture<'a, Result<(WebSocketSink, WebSocketStream), Error>> {
        Box::pin(async move {
            let mode: ConnectionMode = match proxy {
                #[cfg(not(target_arch = "wasm32"))]
                Some(proxy) => ConnectionMode::Proxy(proxy),
                #[cfg(target_arch = "wasm32")]
                Some(_) => ConnectionMode::Direct,
                None => ConnectionMode::Direct,
            };

            // Connect
            let socket: WebSocket = WebSocket::connect(url, &mode)
                .await
                .map_err(Error::transport)?;

            // Split sink and stream
            let (tx, rx) = socket.split();

            // NOTE: don't use sink_map_err here, as it may cause panics!
            // Issue: https://github.com/rust-nostr/nostr/issues/984
            let sink: WebSocketSink = Box::pin(TransportSink(tx)) as WebSocketSink;
            let stream: WebSocketStream = Box::pin(rx.map_err(Error::transport)) as WebSocketStream;

            Ok((sink, stream))
        })
    }
}

struct TransportSink(SplitSink<WebSocket, Message>);

impl Sink<Message> for TransportSink {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_ready_unpin(cx)
            .map_err(Error::transport)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        Pin::new(&mut self.0)
            .start_send_unpin(item)
            .map_err(Error::transport)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_flush_unpin(cx)
            .map_err(Error::transport)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0)
            .poll_close_unpin(cx)
            .map_err(Error::transport)
    }
}
