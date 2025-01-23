// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! WebSocket transport

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_wsocket::futures_util::{self, SinkExt, StreamExt, TryStreamExt};
use async_wsocket::{ConnectionMode, WebSocket, WsMessage};
use nostr::util::BoxedFuture;
use nostr::Url;

use super::error::TransportError;

/// WebSocket transport sink
#[cfg(not(target_arch = "wasm32"))]
pub type Sink = Box<dyn futures_util::Sink<WsMessage, Error = TransportError> + Send + Unpin>;
/// WebSocket transport stream
#[cfg(not(target_arch = "wasm32"))]
pub type Stream =
    Box<dyn futures_util::Stream<Item = Result<WsMessage, TransportError>> + Send + Unpin>;
/// WebSocket transport sink
#[cfg(target_arch = "wasm32")]
pub type Sink = Box<dyn futures_util::Sink<WsMessage, Error = TransportError> + Unpin>;
/// WebSocket transport stream
#[cfg(target_arch = "wasm32")]
pub type Stream = Box<dyn futures_util::Stream<Item = Result<WsMessage, TransportError>> + Unpin>;

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
    ) -> BoxedFuture<'a, Result<(Sink, Stream), TransportError>>;
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
    ) -> BoxedFuture<'a, Result<(Sink, Stream), TransportError>> {
        Box::pin(async move {
            // Connect
            let socket: WebSocket = async_wsocket::connect(url, mode, timeout)
                .await
                .map_err(TransportError::backend)?;

            // Split sink and stream
            let (tx, rx) = socket.split();
            let sink: Sink = Box::new(tx.sink_map_err(TransportError::backend)) as Sink;
            let stream: Stream = Box::new(rx.map_err(TransportError::backend)) as Stream;
            Ok((sink, stream))
        })
    }
}
