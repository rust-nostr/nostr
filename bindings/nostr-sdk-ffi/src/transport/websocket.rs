// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_wsocket::message::{CloseFrame, Message};
use nostr_sdk::pool::transport::websocket::{BoxSink, BoxStream};
use uniffi::{Enum, Object, Record};

use crate::error::Result;
use crate::relay::ConnectionMode;

#[derive(Debug, Record)]
pub struct WebSocketCloseFrame {
    pub code: u16,
    pub reason: String,
}

impl From<WebSocketCloseFrame> for CloseFrame {
    fn from(frame: WebSocketCloseFrame) -> Self {
        Self {
            code: frame.code,
            reason: frame.reason,
        }
    }
}

impl From<CloseFrame> for WebSocketCloseFrame {
    fn from(frame: CloseFrame) -> Self {
        Self {
            code: frame.code,
            reason: frame.reason,
        }
    }
}

#[derive(Debug, Enum)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<WebSocketCloseFrame>),
}

impl From<WebSocketMessage> for Message {
    fn from(msg: WebSocketMessage) -> Self {
        match msg {
            WebSocketMessage::Text(text) => Message::Text(text),
            WebSocketMessage::Binary(binary) => Message::Binary(binary),
            WebSocketMessage::Ping(payload) => Message::Ping(payload),
            WebSocketMessage::Pong(payload) => Message::Pong(payload),
            WebSocketMessage::Close(frame) => Message::Close(frame.map(|f| f.into())),
        }
    }
}

impl From<Message> for WebSocketMessage {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(val) => Self::Text(val),
            Message::Binary(val) => Self::Binary(val),
            Message::Ping(val) => Self::Ping(val),
            Message::Pong(val) => Self::Pong(val),
            Message::Close(frame) => Self::Close(frame.map(|f| f.into())),
        }
    }
}

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait WebSocketAdapter: Send + Sync {
    /// Send a WebSocket message
    async fn send(&self, msg: WebSocketMessage) -> Result<()>;

    /// Receive a message
    ///
    /// This method MUST await for a message.
    ///
    /// Return `None` to mark the stream as terminated.
    async fn recv(&self) -> Result<Option<WebSocketMessage>>;

    /// Close the WebSocket connection
    // Note: `close` method name can't be used in Kotlin
    async fn close_connection(&self) -> Result<()>;
}

#[derive(Object)]
pub struct WebSocketAdapterWrapper {
    inner: Arc<dyn WebSocketAdapter>,
}

#[uniffi::export]
impl WebSocketAdapterWrapper {
    #[uniffi::constructor]
    pub fn new(adapter: Arc<dyn WebSocketAdapter>) -> Self {
        Self { inner: adapter }
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
    ) -> Result<Option<Arc<WebSocketAdapterWrapper>>>;
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

    use async_wsocket::futures_util::{Sink as SinkTrait, Stream as StreamTrait, StreamExt};
    use async_wsocket::ConnectionMode;
    use nostr::util::BoxedFuture;
    use nostr::Url;
    use nostr_sdk::pool::transport::error::TransportError;
    use nostr_sdk::pool::transport::websocket::WebSocketTransport;

    use super::*;
    use crate::error::MiddleError;

    // TODO: use ReusableBoxFuture by tokio-util to avoid reallocation?
    type SinkFuture = Pin<Box<dyn Future<Output = Result<(), TransportError>> + Send>>;
    type StreamFuture =
        Pin<Box<dyn Future<Output = Result<Option<WebSocketMessage>, TransportError>> + Send>>;

    struct FFI2RustWebSocketAdapter {
        // The adapter
        adapter: Arc<dyn WebSocketAdapter>,
        // The messages buffer
        buffer: VecDeque<WebSocketMessage>,
        // Future to flush all messages
        send_all_future: Option<SinkFuture>,
        // Future to close websocket
        close_future: Option<SinkFuture>,
        // Future to recv messages
        recv_future: Option<StreamFuture>,
    }

    impl FFI2RustWebSocketAdapter {
        fn new(adapter: Arc<dyn WebSocketAdapter>) -> Self {
            Self {
                adapter,
                buffer: VecDeque::new(),
                send_all_future: None,
                close_future: None,
                recv_future: None,
            }
        }
    }

    impl SinkTrait<Message> for FFI2RustWebSocketAdapter {
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

        fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
            tracing::trace!("buffering message: {item:?}");
            self.as_mut().buffer.push_back(item.into());
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
            let adapter = this.adapter.clone();

            tracing::trace!("flushing buffered messages: {:?}", messages);

            // Create a future to send all messages
            let future = async move {
                for msg in messages.into_iter() {
                    adapter
                        .send(msg)
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
            // If there's an active future for closing, poll it
            if let Some(close_future) = self.as_mut().close_future.as_mut() {
                return match close_future.as_mut().poll(cx) {
                    // Close complete, clear the future
                    Poll::Ready(result) => {
                        tracing::trace!("poll close completed");
                        self.as_mut().close_future = None;
                        Poll::Ready(result)
                    }
                    Poll::Pending => {
                        tracing::trace!("poll close pending");
                        Poll::Pending
                    }
                };
            }

            // Ensure all buffered messages are flushed before closing
            if self.as_mut().poll_flush(cx).is_pending() {
                return Poll::Pending;
            }

            let mut this = self.as_mut();
            let adapter = this.adapter.clone();

            tracing::trace!("starting poll close");

            // Create a future
            let future = async move {
                adapter
                    .close_connection()
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

    impl StreamTrait for FFI2RustWebSocketAdapter {
        type Item = Result<Message, TransportError>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let mut this = self.as_mut();

            // If there's an active future for closing, poll it
            if let Some(recv_future) = this.recv_future.as_mut() {
                return match recv_future.as_mut().poll(cx) {
                    // Close complete, clear the future
                    Poll::Ready(result) => {
                        tracing::trace!("poll next completed");
                        this.recv_future = None;

                        // Convert output
                        let output: Option<Self::Item> = match result {
                            Ok(Some(msg)) => Some(Ok(msg.into())),
                            Ok(None) => None,
                            Err(e) => Some(Err(e)),
                        };

                        // Poll
                        Poll::Ready(output)
                    }
                    Poll::Pending => {
                        tracing::trace!("poll next pending");
                        Poll::Pending
                    }
                };
            }

            let adapter = this.adapter.clone();

            tracing::trace!("starting poll next");

            // Create a future
            let future = async move {
                adapter
                    .recv()
                    .await
                    .map_err(MiddleError::from)
                    .map_err(TransportError::backend)
            };

            // Store this future in the state
            this.recv_future = Some(Box::pin(future));

            // Start polling the future
            this.poll_next(cx)
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
        ) -> BoxedFuture<'a, Result<(BoxSink, BoxStream), TransportError>> {
            Box::pin(async move {
                let intermediate = self
                    .inner
                    .connect(url.to_string(), mode.clone().into(), timeout)
                    .await
                    .map_err(|e| TransportError::backend(MiddleError::from(e)))?
                    .ok_or_else(|| {
                        TransportError::backend(MiddleError::new("WebSocket adapter not found"))
                    })?;

                // Construct socket
                let socket = FFI2RustWebSocketAdapter::new(intermediate.inner.clone());

                // Split it
                let (tx, rx) = socket.split();

                let sink: BoxSink = Box::new(tx) as BoxSink;
                let stream: BoxStream = Box::new(rx) as BoxStream;

                Ok((sink, stream))
            })
        }
    }
}
