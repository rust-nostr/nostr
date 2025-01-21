// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_wsocket::WsMessage;
use nostr_sdk::async_utility::tokio::sync::{mpsc, Mutex};
use nostr_sdk::pool::transport::websocket::{Sink, Stream};
use uniffi::{Enum, Object};

use crate::error::{NostrSdkError, Result};
use crate::relay::ConnectionMode;

#[derive(Debug, Enum)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

impl From<WebSocketMessage> for WsMessage {
    fn from(msg: WebSocketMessage) -> Self {
        match msg {
            WebSocketMessage::Text(text) => WsMessage::Text(text),
            WebSocketMessage::Binary(binary) => WsMessage::Binary(binary),
            WebSocketMessage::Ping(payload) => WsMessage::Ping(payload),
            WebSocketMessage::Pong(payload) => WsMessage::Pong(payload),
        }
    }
}

impl TryFrom<WsMessage> for WebSocketMessage {
    type Error = NostrSdkError;

    fn try_from(msg: WsMessage) -> Result<Self> {
        match msg {
            WsMessage::Text(val) => Ok(Self::Text(val)),
            WsMessage::Binary(val) => Ok(Self::Binary(val)),
            WsMessage::Ping(val) => Ok(Self::Ping(val)),
            WsMessage::Pong(val) => Ok(Self::Pong(val)),
            _ => Err(NostrSdkError::Generic(String::from(
                "unsupported message type",
            ))),
        }
    }
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait WebSocketSink: Send + Sync {
    /// Send a WebSocket message
    async fn send_msg(&self, msg: WebSocketMessage) -> Result<()>;

    /// Close the WebSocket connection
    async fn terminate(&self) -> Result<()>;
}

#[derive(Object)]
pub struct WebSocketStreamForwarder {
    tx: mpsc::Sender<WsMessage>,
    rx: Arc<Mutex<Option<mpsc::Receiver<WsMessage>>>>,
}

#[uniffi::export(async_runtime = "tokio")]
impl WebSocketStreamForwarder {
    /// Create new WebSocket stream forwarder
    ///
    /// NOTE: for every connection must be created a new forwarder!
    #[uniffi::constructor]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(4096);
        Self {
            tx,
            rx: Arc::new(Mutex::new(Some(rx))),
        }
    }

    /// Forward the message to the ingestor.
    pub async fn forward(&self, msg: WebSocketMessage) -> Result<()> {
        self.tx.send(msg.into()).await?;
        Ok(())
    }
}

#[derive(Object)]
pub struct WebSocketAdaptor {
    sink: Arc<dyn WebSocketSink>,
    stream: Arc<WebSocketStreamForwarder>,
}

#[uniffi::export]
impl WebSocketAdaptor {
    #[uniffi::constructor]
    pub fn new(sink: Arc<dyn WebSocketSink>, stream: Arc<WebSocketStreamForwarder>) -> Self {
        Self { sink, stream }
    }
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait CustomWebSocketTransport: Send + Sync {
    /// If returns `true`, the WebSocket implementation must handle and forward the PING/PONG messages.
    /// The ping is used by the SDK,
    /// for example, to calculate the average latency or to make sure the relay is still connected.
    fn support_ping(&self) -> bool;

    /// Connect to a relay
    async fn connect(
        &self,
        url: String,
        mode: ConnectionMode,
        timeout: Duration,
    ) -> Result<Option<Arc<WebSocketAdaptor>>>;
}

pub(crate) struct FFI2RustWebSocketTransport {
    pub(crate) inner: Arc<dyn CustomWebSocketTransport>,
}

impl fmt::Debug for FFI2RustWebSocketTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FFI2RustWebSocketTransport").finish()
    }
}

mod inner {
    use std::collections::VecDeque;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use async_wsocket::futures_util::{Sink as SinkTrait, StreamExt};
    use async_wsocket::ConnectionMode;
    use nostr::util::BoxedFuture;
    use nostr::Url;
    use nostr_sdk::pool::stream::ReceiverStream;
    use nostr_sdk::pool::transport::error::TransportError;
    use nostr_sdk::pool::transport::websocket::WebSocketTransport;

    use super::*;
    use crate::error::MiddleError;

    type SinkFuture = Pin<Box<dyn Future<Output = Result<(), TransportError>> + Send>>;

    struct FFI2RustWebSocketSink {
        inner: Arc<dyn WebSocketSink>,
        buffer: VecDeque<WebSocketMessage>,
        // Future to flush all messages
        send_all_future: Option<SinkFuture>,
        // Future to close websocket
        close_future: Option<SinkFuture>,
    }

    impl FFI2RustWebSocketSink {
        fn new(sink: Arc<dyn WebSocketSink>) -> Self {
            Self {
                inner: sink,
                buffer: VecDeque::new(),
                send_all_future: None,
                close_future: None,
            }
        }
    }

    impl SinkTrait<WsMessage> for FFI2RustWebSocketSink {
        type Error = TransportError;

        fn poll_ready(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            if self.buffer.is_empty() && self.send_all_future.is_none() {
                Poll::Ready(Ok(()))
            } else {
                // The buffer must be flushed or the future not completed yet.
                Poll::Pending
            }
        }

        fn start_send(mut self: Pin<&mut Self>, item: WsMessage) -> Result<(), Self::Error> {
            tracing::trace!("buffering message: {item:?}");
            let msg: WebSocketMessage = item
                .try_into()
                .map_err(|e| TransportError::backend(MiddleError::from(e)))?;
            self.as_mut().buffer.push_back(msg);
            Ok(())
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            let mut this = self.as_mut();

            // If there's an active future for sending all messages, poll it
            if let Some(send_future) = this.send_all_future.as_mut() {
                return match send_future.as_mut().poll(cx) {
                    // Sending complete, clear the future
                    Poll::Ready(result) => {
                        tracing::trace!("flushing completed");
                        this.send_all_future = None;
                        Poll::Ready(result)
                    }
                    Poll::Pending => {
                        tracing::trace!("flushing pending");
                        Poll::Pending
                    }
                };
            }

            // No active future exists and nothing to flush
            if this.buffer.is_empty() {
                tracing::trace!("flushing completed");

                // Nothing to flush, return Ready
                return Poll::Ready(Ok(()));
            }

            // Take buffer
            let messages: VecDeque<WebSocketMessage> = std::mem::take(&mut this.buffer);
            let sink = this.inner.clone();

            tracing::trace!("flushing buffered messages: {:?}", messages);

            // Create a future to send all messages
            let future = async move {
                for msg in messages.into_iter() {
                    sink.send_msg(msg)
                        .await
                        .map_err(MiddleError::from)
                        .map_err(TransportError::backend)?;
                }
                Ok(())
            };

            // Store this future in the state
            this.send_all_future = Some(Box::pin(future));

            // Start polling the future
            this.poll_flush(cx)
        }

        fn poll_close(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            // Ensure all buffered messages are flushed before closing
            if self.as_mut().poll_flush(cx).is_pending() {
                return Poll::Pending;
            }

            let mut this = self.as_mut();

            // If there's an active future for closing, poll it
            if let Some(close_future) = this.close_future.as_mut() {
                return match close_future.as_mut().poll(cx) {
                    // Close complete, clear the future
                    Poll::Ready(result) => {
                        tracing::trace!("poll close completed");
                        this.close_future = None;
                        Poll::Ready(result)
                    }
                    Poll::Pending => {
                        tracing::trace!("poll close pending");
                        Poll::Pending
                    }
                };
            }

            let sink = this.inner.clone();

            tracing::trace!("starting poll close");

            // Create a future to send all messages
            let future = async move {
                sink.terminate()
                    .await
                    .map_err(MiddleError::from)
                    .map_err(TransportError::backend)
            };

            // Store this future in the state
            this.close_future = Some(Box::pin(future));

            // Start polling the future
            this.poll_close(cx)
        }
    }

    impl WebSocketTransport for FFI2RustWebSocketTransport {
        fn support_ping(&self) -> bool {
            self.inner.support_ping()
        }

        fn connect<'a>(
            &'a self,
            url: &'a Url,
            mode: &'a ConnectionMode,
            timeout: Duration,
        ) -> BoxedFuture<'a, Result<(Sink, Stream), TransportError>> {
            Box::pin(async move {
                let intermediate = self
                    .inner
                    .connect(url.to_string(), mode.clone().into(), timeout)
                    .await
                    .map_err(|e| TransportError::backend(MiddleError::from(e)))?
                    .ok_or_else(|| {
                        TransportError::backend(MiddleError::new("WebSocket adaptor not found"))
                    })?;

                let sink: Sink = Box::new(FFI2RustWebSocketSink::new(intermediate.sink.clone()));

                let stream: Stream = {
                    let mut rx = intermediate.stream.rx.lock().await;
                    let inner = rx.take().ok_or_else(|| {
                        TransportError::backend(MiddleError::new("receiver already taken"))
                    })?;
                    drop(rx);
                    Box::new(ReceiverStream::new(inner).map(Ok)) as Stream
                };

                Ok((sink, stream))
            })
        }
    }
}
