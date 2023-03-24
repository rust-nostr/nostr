// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Native

use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::future::Ready;
use futures_util::stream::{FilterMap, SplitSink, SplitStream, StreamExt};
use futures_util::{Sink, Stream};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName};
use tokio_rustls::TlsConnector;
use tokio_tungstenite::tungstenite::error::{
    CapacityError as WsCapacityError, Error as WsError, ProtocolError as WsProtocolError,
    UrlError as WsUrlError,
};
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Message as WsMessage};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream as WsStream};
use url::{ParseError, Url};

mod socks;

use self::socks::TpcSocks5Stream;
use crate::error::{CapacityError, Error, ProtocolError, Result, UrlError};
use crate::message::Message;

type WebSocket = WsStream<MaybeTlsStream<TcpStream>>;
type MsgConvFut = Ready<Option<Result<Message>>>;

#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    #[error("socks error: {0}")]
    Socks(#[from] tokio_socks::Error),
    /// Timeout
    #[error("timeout")]
    Timeout,
    /// Invalid DNS name
    #[error("invalid DNS name")]
    InvalidDNSName,
    /// Url parse error
    #[error("impossible to parse URL: {0}")]
    Url(#[from] url::ParseError),
}

pub async fn connect(
    url: &Url,
    proxy: Option<SocketAddr>,
    timeout: Option<Duration>,
) -> Result<
    (
        SplitSink<WebSocketStream, Message>,
        SplitStream<WebSocketStream>,
    ),
    Error,
> {
    let stream = match proxy {
        Some(proxy) => connect_proxy(url, proxy, timeout).await?,
        None => connect_direct(url, timeout).await?,
    };
    let ws = WebSocketStream {
        inner: stream.filter_map(msg_conv),
    };
    Ok(ws.split())
}

async fn connect_direct(url: &Url, timeout: Option<Duration>) -> Result<WebSocket, Error> {
    let timeout = timeout.unwrap_or(Duration::from_secs(60));
    let (stream, _) = tokio::time::timeout(timeout, tokio_tungstenite::connect_async(url))
        .await
        .map_err(|_| NativeError::Timeout)??;
    Ok(stream)
}

async fn connect_proxy(
    url: &Url,
    proxy: SocketAddr,
    timeout: Option<Duration>,
) -> Result<WebSocket, Error> {
    let timeout = timeout.unwrap_or(Duration::from_secs(60));
    let addr: String = match url.host_str() {
        Some(host) => match url.port_or_known_default() {
            Some(port) => format!("{host}:{port}"),
            None => return Err(NativeError::Url(ParseError::EmptyHost).into()),
        },
        None => return Err(NativeError::Url(ParseError::InvalidPort).into()),
    };

    // TODO: remove these unwrap
    let conn = TpcSocks5Stream::connect(proxy, addr.clone()).await.unwrap();
    let conn = match connect_with_tls(conn, url).await {
        Ok(stream) => MaybeTlsStream::Rustls(stream),
        Err(_) => {
            let conn = TpcSocks5Stream::connect(proxy, addr).await.unwrap();
            MaybeTlsStream::Plain(conn)
        }
    };

    let (stream, _) = tokio::time::timeout(timeout, tokio_tungstenite::client_async(url, conn))
        .await
        .map_err(|_| NativeError::Timeout)??;
    Ok(stream)
}

async fn connect_with_tls(stream: TcpStream, url: &Url) -> Result<TlsStream<TcpStream>, Error> {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let domain = url.domain().ok_or(NativeError::InvalidDNSName)?;
    let domain = ServerName::try_from(domain).map_err(|_| NativeError::InvalidDNSName)?;
    Ok(connector.connect(domain, stream).await?)
}

#[allow(clippy::type_complexity)]
pub struct WebSocketStream {
    inner: FilterMap<WebSocket, MsgConvFut, fn(Result<WsMessage, WsError>) -> MsgConvFut>,
}

impl Stream for WebSocketStream {
    type Item = Result<crate::Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl Sink<crate::Message> for WebSocketStream {
    type Error = Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx).map_err(Into::into)
    }

    fn start_send(
        mut self: Pin<&mut Self>,
        item: crate::Message,
    ) -> std::result::Result<(), Self::Error> {
        Pin::new(&mut self.inner)
            .start_send(item.into())
            .map_err(Into::into)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx).map_err(Into::into)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx).map_err(Into::into)
    }
}

fn msg_conv(msg: Result<WsMessage, WsError>) -> MsgConvFut {
    fn inner(msg: Result<WsMessage, WsError>) -> Option<Result<Message>> {
        let msg = match msg {
            Ok(msg) => match msg {
                WsMessage::Text(inner) => Ok(Message::Text(inner)),
                WsMessage::Binary(inner) => Ok(Message::Binary(inner)),
                WsMessage::Ping(inner) => Ok(Message::Ping(inner)),
                WsMessage::Pong(inner) => Ok(Message::Pong(inner)),
                WsMessage::Close(inner) => Ok(Message::Close(inner.map(Into::into))),
                WsMessage::Frame(inner) => Ok(Message::Frame(inner)),
            },
            Err(err) => Err(Error::from(err)),
        };
        Some(msg)
    }
    futures_util::future::ready(inner(msg))
}

impl<'a> From<CloseFrame<'a>> for crate::message::CloseFrame<'a> {
    fn from(close_frame: CloseFrame<'a>) -> Self {
        crate::message::CloseFrame {
            code: u16::from(close_frame.code).into(),
            reason: close_frame.reason,
        }
    }
}

impl<'a> From<crate::message::CloseFrame<'a>> for CloseFrame<'a> {
    fn from(close_frame: crate::message::CloseFrame<'a>) -> Self {
        CloseFrame {
            code: u16::from(close_frame.code).into(),
            reason: close_frame.reason,
        }
    }
}

impl From<WsMessage> for Message {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Text(inner) => Message::Text(inner),
            WsMessage::Binary(inner) => Message::Binary(inner),
            WsMessage::Ping(bytes) => Message::Ping(bytes),
            WsMessage::Pong(bytes) => Message::Pong(bytes),
            WsMessage::Close(inner) => Message::Close(inner.map(Into::into)),
            WsMessage::Frame(inner) => Message::Frame(inner),
        }
    }
}

impl From<Message> for WsMessage {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(inner) => WsMessage::Text(inner),
            Message::Binary(inner) => WsMessage::Binary(inner),
            Message::Ping(bytes) => WsMessage::Ping(bytes),
            Message::Pong(bytes) => WsMessage::Pong(bytes),
            Message::Close(inner) => WsMessage::Close(inner.map(Into::into)),
            Message::Frame(inner) => WsMessage::Frame(inner),
        }
    }
}

impl From<WsError> for Error {
    fn from(err: WsError) -> Self {
        match err {
            WsError::ConnectionClosed => Error::ConnectionClosed,
            WsError::AlreadyClosed => Error::AlreadyClosed,
            WsError::Io(inner) => Error::Io(inner),
            WsError::Tls(inner) => Error::Tls(inner),
            WsError::Capacity(inner) => Error::Capacity(inner.into()),
            WsError::Protocol(inner) => Error::Protocol(inner.into()),
            WsError::SendQueueFull(inner) => Error::SendQueueFull(inner.into()),
            WsError::Utf8 => Error::Utf8,
            WsError::Url(inner) => Error::Url(inner.into()),
            WsError::Http(inner) => Error::Http(inner),
            WsError::HttpFormat(inner) => Error::HttpFormat(inner),
        }
    }
}

impl From<WsCapacityError> for CapacityError {
    fn from(err: WsCapacityError) -> Self {
        match err {
            WsCapacityError::TooManyHeaders => CapacityError::TooManyHeaders,
            WsCapacityError::MessageTooLong { size, max_size } => {
                CapacityError::MessageTooLong { size, max_size }
            }
        }
    }
}

impl From<WsUrlError> for UrlError {
    fn from(err: WsUrlError) -> Self {
        match err {
            WsUrlError::TlsFeatureNotEnabled => UrlError::TlsFeatureNotEnabled,
            WsUrlError::NoHostName => UrlError::NoHostName,
            WsUrlError::UnableToConnect(inner) => UrlError::UnableToConnect(inner),
            WsUrlError::UnsupportedUrlScheme => UrlError::UnsupportedUrlScheme,
            WsUrlError::EmptyHostName => UrlError::EmptyHostName,
            WsUrlError::NoPathOrQuery => UrlError::NoPathOrQuery,
        }
    }
}

impl From<WsProtocolError> for ProtocolError {
    fn from(err: WsProtocolError) -> Self {
        match err {
            WsProtocolError::WrongHttpMethod => ProtocolError::WrongHttpMethod,
            WsProtocolError::WrongHttpVersion => ProtocolError::WrongHttpVersion,
            WsProtocolError::MissingConnectionUpgradeHeader => {
                ProtocolError::MissingConnectionUpgradeHeader
            }
            WsProtocolError::MissingUpgradeWebSocketHeader => {
                ProtocolError::MissingUpgradeWebSocketHeader
            }
            WsProtocolError::MissingSecWebSocketVersionHeader => {
                ProtocolError::MissingSecWebSocketVersionHeader
            }
            WsProtocolError::MissingSecWebSocketKey => ProtocolError::MissingSecWebSocketKey,
            WsProtocolError::SecWebSocketAcceptKeyMismatch => {
                ProtocolError::SecWebSocketAcceptKeyMismatch
            }
            WsProtocolError::JunkAfterRequest => ProtocolError::JunkAfterRequest,
            WsProtocolError::CustomResponseSuccessful => ProtocolError::CustomResponseSuccessful,
            WsProtocolError::HandshakeIncomplete => ProtocolError::HandshakeIncomplete,
            WsProtocolError::HttparseError(inner) => ProtocolError::HttparseError(inner),
            WsProtocolError::SendAfterClosing => ProtocolError::SendAfterClosing,
            WsProtocolError::ReceivedAfterClosing => ProtocolError::ReceivedAfterClosing,
            WsProtocolError::NonZeroReservedBits => ProtocolError::NonZeroReservedBits,
            WsProtocolError::UnmaskedFrameFromClient => ProtocolError::UnmaskedFrameFromClient,
            WsProtocolError::MaskedFrameFromServer => ProtocolError::MaskedFrameFromServer,
            WsProtocolError::FragmentedControlFrame => ProtocolError::FragmentedControlFrame,
            WsProtocolError::ControlFrameTooBig => ProtocolError::ControlFrameTooBig,
            WsProtocolError::UnknownControlFrameType(inner) => {
                ProtocolError::UnknownControlFrameType(inner)
            }
            WsProtocolError::UnknownDataFrameType(inner) => {
                ProtocolError::UnknownDataFrameType(inner)
            }
            WsProtocolError::UnexpectedContinueFrame => ProtocolError::UnexpectedContinueFrame,
            WsProtocolError::ExpectedFragment(inner) => ProtocolError::ExpectedFragment(inner),
            WsProtocolError::ResetWithoutClosingHandshake => {
                ProtocolError::ResetWithoutClosingHandshake
            }
            WsProtocolError::InvalidOpcode(inner) => ProtocolError::InvalidOpcode(inner),
            WsProtocolError::InvalidCloseSequence => ProtocolError::InvalidCloseSequence,
            WsProtocolError::InvalidHeader(inner) => ProtocolError::InvalidHeader(inner),
        }
    }
}
