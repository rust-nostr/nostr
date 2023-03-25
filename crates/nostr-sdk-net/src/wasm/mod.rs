// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Web

use std::cell::RefCell;
use std::collections::VecDeque;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use futures_util::stream::StreamExt;
use futures_util::stream::{SplitSink, SplitStream};
use url::Url;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

pub mod error;
pub mod message;

use self::error::{Error, Result, UrlError};
use self::message::{CloseFrame, Message};

type Sink = SplitSink<WebSocketStream, Message>;
type Stream = SplitStream<WebSocketStream>;

pub async fn connect(url: &Url) -> Result<(Sink, Stream)> {
    let stream = WebSocketStream::new(url).await?;
    Ok(stream.split())
}

pub struct WebSocketStream {
    inner: WebSocket,
    queue: Rc<RefCell<VecDeque<Result<Message>>>>,
    waker: Rc<RefCell<Option<Waker>>>,
    _on_message_callback: Closure<dyn FnMut(MessageEvent)>,
    _on_error_callback: Closure<dyn FnMut(ErrorEvent)>,
    _on_close_callback: Closure<dyn FnMut(CloseEvent)>,
}

impl WebSocketStream {
    async fn new(url: &Url) -> Result<Self> {
        match web_sys::WebSocket::new(url.to_string().as_str()) {
            Ok(ws) => {
                ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

                let (open_sx, open_rx) = futures_channel::oneshot::channel();
                let on_open_callback = {
                    let mut open_sx = Some(open_sx);
                    Closure::wrap(Box::new(move |_event| {
                        open_sx.take().map(|open_sx| open_sx.send(()));
                    }) as Box<dyn FnMut(web_sys::Event)>)
                };
                ws.set_onopen(Some(on_open_callback.as_ref().unchecked_ref()));

                let (err_sx, err_rx) = futures_channel::oneshot::channel();
                let on_error_callback = {
                    let mut err_sx = Some(err_sx);
                    Closure::wrap(Box::new(move |_error_event| {
                        err_sx.take().map(|err_sx| err_sx.send(()));
                    }) as Box<dyn FnMut(ErrorEvent)>)
                };
                ws.set_onerror(Some(on_error_callback.as_ref().unchecked_ref()));

                let result = futures_util::future::select(open_rx, err_rx).await;
                ws.set_onopen(None);
                ws.set_onerror(None);
                let ws = match result {
                    futures_util::future::Either::Left((_, _)) => Ok(ws),
                    futures_util::future::Either::Right((_, _)) => Err(Error::ConnectionClosed),
                }?;

                let waker = Rc::new(RefCell::new(Option::<Waker>::None));
                let queue = Rc::new(RefCell::new(VecDeque::new()));
                let on_message_callback = {
                    let waker = Rc::clone(&waker);
                    let queue = Rc::clone(&queue);
                    Closure::wrap(Box::new(move |event: MessageEvent| {
                        let payload = std::convert::TryFrom::try_from(event);
                        queue.borrow_mut().push_back(payload);
                        if let Some(waker) = waker.borrow_mut().take() {
                            waker.wake();
                        }
                    }) as Box<dyn FnMut(MessageEvent)>)
                };
                ws.set_onmessage(Some(on_message_callback.as_ref().unchecked_ref()));

                let on_error_callback = {
                    let waker = Rc::clone(&waker);
                    let queue = Rc::clone(&queue);
                    Closure::wrap(Box::new(move |_error_event| {
                        queue.borrow_mut().push_back(Err(Error::ConnectionClosed));
                        if let Some(waker) = waker.borrow_mut().take() {
                            waker.wake();
                        }
                    }) as Box<dyn FnMut(ErrorEvent)>)
                };
                ws.set_onerror(Some(on_error_callback.as_ref().unchecked_ref()));

                let on_close_callback = {
                    let waker = Rc::clone(&waker);
                    let queue = Rc::clone(&queue);
                    Closure::wrap(Box::new(move |event: CloseEvent| {
                        queue
                            .borrow_mut()
                            .push_back(Ok(Message::Close(Some(CloseFrame {
                                code: event.code().into(),
                                reason: event.reason().into(),
                            }))));
                        if let Some(waker) = waker.borrow_mut().take() {
                            waker.wake();
                        }
                    }) as Box<dyn FnMut(CloseEvent)>)
                };
                ws.set_onclose(Some(on_close_callback.as_ref().unchecked_ref()));

                Ok(Self {
                    inner: ws,
                    queue,
                    waker,
                    _on_message_callback: on_message_callback,
                    _on_error_callback: on_error_callback,
                    _on_close_callback: on_close_callback,
                })
            }
            Err(_) => Err(Error::Url(UrlError::UnsupportedUrlScheme)),
        }
    }
}

impl Drop for WebSocketStream {
    fn drop(&mut self) {
        let _r = self.inner.close();
        self.inner.set_onmessage(None);
        self.inner.set_onclose(None);
        self.inner.set_onerror(None);
    }
}

enum ReadyState {
    Closed,
    Closing,
    Connecting,
    Open,
}

impl std::convert::TryFrom<u16> for ReadyState {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            web_sys::WebSocket::CLOSED => Ok(Self::Closed),
            web_sys::WebSocket::CLOSING => Ok(Self::Closing),
            web_sys::WebSocket::OPEN => Ok(Self::Open),
            web_sys::WebSocket::CONNECTING => Ok(Self::Connecting),
            _ => Err(()),
        }
    }
}

impl futures_util::Stream for WebSocketStream {
    type Item = Result<Message>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.queue.borrow().is_empty() {
            *self.waker.borrow_mut() = Some(cx.waker().clone());

            match std::convert::TryFrom::try_from(self.inner.ready_state()) {
                Ok(ReadyState::Open) => Poll::Pending,
                _ => None.into(),
            }
        } else {
            self.queue.borrow_mut().pop_front().into()
        }
    }
}

impl futures_util::Sink<Message> for WebSocketStream {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match std::convert::TryFrom::try_from(self.inner.ready_state()) {
            Ok(ReadyState::Open) => Ok(()).into(),
            _ => Err(Error::ConnectionClosed).into(),
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match std::convert::TryFrom::try_from(self.inner.ready_state()) {
            Ok(ReadyState::Open) => {
                match item {
                    Message::Text(text) => {
                        self.inner.send_with_str(&text).map_err(|_| Error::Utf8)?
                    }
                    Message::Binary(data) => self
                        .inner
                        .send_with_u8_array(&data)
                        .map_err(|_| Error::Utf8)?,
                    Message::Ping(data) => self
                        .inner
                        .send_with_u8_array(&data)
                        .map_err(|_| Error::Utf8)?,
                    Message::Pong(data) => self
                        .inner
                        .send_with_u8_array(&data)
                        .map_err(|_| Error::Utf8)?,
                    Message::Close(frame) => match frame {
                        None => self.inner.close().map_err(|_| Error::AlreadyClosed)?,
                        Some(frame) => self
                            .inner
                            .close_with_code_and_reason(frame.code.into(), &frame.reason)
                            .map_err(|_| Error::AlreadyClosed)?,
                    },
                }
                Ok(())
            }
            _ => Err(Error::ConnectionClosed),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.close().map_err(|_| Error::AlreadyClosed)?;
        Ok(()).into()
    }
}

impl std::convert::TryFrom<web_sys::MessageEvent> for Message {
    type Error = Error;

    fn try_from(event: MessageEvent) -> Result<Self, Self::Error> {
        match event.data() {
            payload if payload.is_instance_of::<js_sys::ArrayBuffer>() => {
                let buffer = js_sys::Uint8Array::new(payload.unchecked_ref());
                let mut v = vec![0; buffer.length() as usize];
                buffer.copy_to(&mut v);
                Ok(Message::Binary(v))
            }
            payload if payload.is_string() => match payload.as_string() {
                Some(text) => Ok(Message::Text(text)),
                None => Err(Error::Utf8),
            },
            payload if payload.is_instance_of::<web_sys::Blob>() => {
                Err(Error::BlobFormatUnsupported)
            }
            _ => Err(Error::UnknownFormat),
        }
    }
}
