// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::SocketAddr;
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use nostr::url::{ParseError, Url};
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WebSocketSocks5 = WebSocketStream<Socks5Stream<TcpStream>>;

type SplitSinkDirect = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type StreamClearnet = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

type SplitSinkSocks5 = SplitSink<WebSocketStream<Socks5Stream<TcpStream>>, Message>;
type StreamSocks5 = SplitStream<WebSocketStream<Socks5Stream<TcpStream>>>;

mod socks;

use self::socks::TpcSocks5Stream;

#[derive(Debug)]
pub enum Error {
    /// Ws error
    Ws(WsError),
    Socks(tokio_socks::Error),
    /// Timeout
    Timeout,
    /// Url parse error
    Url(nostr::url::ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ws(err) => write!(f, "ws error: {}", err),
            Self::Socks(err) => write!(f, "socks error: {}", err),
            Self::Timeout => write!(f, "timeout"),
            Self::Url(err) => write!(f, "impossible to parse URL: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<WsError> for Error {
    fn from(err: WsError) -> Self {
        Self::Ws(err)
    }
}

impl From<tokio_socks::Error> for Error {
    fn from(err: tokio_socks::Error) -> Self {
        Self::Socks(err)
    }
}

#[derive(Debug)]
pub(crate) enum Sink {
    Direct(SplitSinkDirect),
    Socks5(SplitSinkSocks5),
}

impl Sink {
    pub async fn send(&mut self, msg: Message) -> Result<(), Error> {
        match self {
            Self::Direct(ws_tx) => Ok(ws_tx.send(msg).await?),
            Self::Socks5(ws_tx) => Ok(ws_tx.send(msg).await?),
        }
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        match self {
            Self::Direct(ws_tx) => Ok(ws_tx.close().await?),
            Self::Socks5(ws_tx) => Ok(ws_tx.close().await?),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Stream {
    Direct(StreamClearnet),
    Socks5(StreamSocks5),
}

impl Stream {
    pub async fn next(&mut self) -> Option<Result<Message, WsError>> {
        match self {
            Self::Direct(ws_rx) => ws_rx.next().await,
            Self::Socks5(ws_rx) => ws_rx.next().await,
        }
    }
}

pub(crate) async fn get_connection(
    url: &Url,
    proxy: Option<SocketAddr>,
    timeout: Option<Duration>,
) -> Result<(Sink, Stream), Error> {
    match proxy {
        Some(proxy) => {
            let stream = connect_proxy(url, proxy, timeout).await?;
            let (sink, stream) = stream.split();
            Ok((Sink::Socks5(sink), Stream::Socks5(stream)))
        }
        None => {
            let stream = connect(url, timeout).await?;
            let (sink, stream) = stream.split();
            Ok((Sink::Direct(sink), Stream::Direct(stream)))
        }
    }
}

async fn connect(url: &Url, timeout: Option<Duration>) -> Result<WebSocket, Error> {
    let timeout = timeout.unwrap_or(Duration::from_secs(60));
    let (stream, _) = tokio::time::timeout(timeout, tokio_tungstenite::connect_async(url))
        .await
        .map_err(|_| Error::Timeout)??;
    Ok(stream)
}

async fn connect_proxy(
    url: &Url,
    proxy: SocketAddr,
    timeout: Option<Duration>,
) -> Result<WebSocketSocks5, Error> {
    let timeout = timeout.unwrap_or(Duration::from_secs(60));
    let addr: String = match url.host_str() {
        Some(host) => match url.port_or_known_default() {
            Some(port) => format!("{}:{}", host, port),
            None => return Err(Error::Url(ParseError::EmptyHost)),
        },
        None => return Err(Error::Url(ParseError::InvalidPort)),
    };

    log::debug!("Addr: {}", addr);

    let conn = TpcSocks5Stream::connect(proxy, addr).await?;
    let (stream, _) = tokio::time::timeout(timeout, tokio_tungstenite::client_async(url, conn))
        .await
        .map_err(|_| Error::Timeout)??;
    Ok(stream)
}
