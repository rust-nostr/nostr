// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

use anyhow::{anyhow, Result};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use url::Url;

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WebSocketSocks5 = WebSocketStream<Socks5Stream<TcpStream>>;

type SplitSinkDirect = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type StreamClearnet = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

type SplitSinkSocks5 = SplitSink<WebSocketStream<Socks5Stream<TcpStream>>, Message>;
type StreamSocks5 = SplitStream<WebSocketStream<Socks5Stream<TcpStream>>>;

use super::socks::TpcSocks5Stream;

#[derive(Debug)]
pub(crate) enum Sink {
    Direct(SplitSinkDirect),
    Socks5(SplitSinkSocks5),
}

impl Sink {
    pub async fn send(&mut self, msg: Message) -> Result<()> {
        match self {
            Self::Direct(ws_tx) => Ok(ws_tx.send(msg).await?),
            Self::Socks5(ws_tx) => Ok(ws_tx.send(msg).await?),
        }
    }

    pub async fn close(&mut self) -> Result<()> {
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
    pub async fn next(&mut self) -> Option<Result<Message, Error>> {
        match self {
            Self::Direct(ws_rx) => ws_rx.next().await,
            Self::Socks5(ws_rx) => ws_rx.next().await,
        }
    }
}

pub(crate) async fn get_connection(url: &Url, proxy: Option<SocketAddr>) -> Result<(Sink, Stream)> {
    match proxy {
        Some(proxy) => {
            let stream = connect_proxy(url, proxy).await?;
            let (sink, stream) = stream.split();
            Ok((Sink::Socks5(sink), Stream::Socks5(stream)))
        }
        None => {
            let stream = connect(url).await?;
            let (sink, stream) = stream.split();
            Ok((Sink::Direct(sink), Stream::Direct(stream)))
        }
    }
}

async fn connect(url: &Url) -> Result<WebSocket> {
    let (stream, _) = tokio_tungstenite::connect_async(url).await?;
    Ok(stream)
}

async fn connect_proxy(url: &Url, proxy: SocketAddr) -> Result<WebSocketSocks5> {
    let addr: String = match url.host_str() {
        Some(host) => match url.port_or_known_default() {
            Some(port) => format!("{}:{}", host, port),
            None => return Err(anyhow!("Impossible to extract port from url")),
        },
        None => return Err(anyhow!("Impossible to extract host from url")),
    };

    log::debug!("Addr: {}", addr);

    let conn = TpcSocks5Stream::connect(proxy, addr).await?;
    let (stream, _response) = tokio_tungstenite::client_async(url, conn).await?;
    Ok(stream)
}
