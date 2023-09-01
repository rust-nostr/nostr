// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Native Network

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName};
use tokio_rustls::TlsConnector;
use tokio_tungstenite::tungstenite::Error as WsError;
pub use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use url_fork::{ParseError, Url};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Sink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type Stream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

mod socks;

use self::socks::TpcSocks5Stream;

#[derive(Debug, Error)]
pub enum Error {
    /// I/O error
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    /// Ws error
    #[error("ws error: {0}")]
    Ws(#[from] WsError),
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
    Url(#[from] ParseError),
}

pub async fn connect(
    url: &Url,
    proxy: Option<SocketAddr>,
    timeout: Option<Duration>,
) -> Result<(Sink, Stream), Error> {
    let stream = match proxy {
        Some(proxy) => connect_proxy(url, proxy, timeout).await?,
        None => connect_direct(url, timeout).await?,
    };
    Ok(stream.split())
}

async fn connect_direct(url: &Url, timeout: Option<Duration>) -> Result<WebSocket, Error> {
    let timeout = timeout.unwrap_or(Duration::from_secs(60));
    let (stream, _) =
        tokio::time::timeout(timeout, tokio_tungstenite::connect_async(url.to_string()))
            .await
            .map_err(|_| Error::Timeout)??;
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
            None => return Err(Error::Url(ParseError::EmptyHost)),
        },
        None => return Err(Error::Url(ParseError::InvalidPort)),
    };

    let conn = TpcSocks5Stream::connect(proxy, addr.clone()).await?;
    let conn = match connect_with_tls(conn, url).await {
        Ok(stream) => MaybeTlsStream::Rustls(stream),
        Err(_) => {
            let conn = TpcSocks5Stream::connect(proxy, addr).await?;
            MaybeTlsStream::Plain(conn)
        }
    };

    let (stream, _) = tokio::time::timeout(
        timeout,
        tokio_tungstenite::client_async(url.to_string(), conn),
    )
    .await
    .map_err(|_| Error::Timeout)??;
    Ok(stream)
}

async fn connect_with_tls(stream: TcpStream, url: &Url) -> Result<TlsStream<TcpStream>, Error> {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
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
    let domain = url.domain().ok_or(Error::InvalidDNSName)?;
    let domain = ServerName::try_from(domain).map_err(|_| Error::InvalidDNSName)?;
    Ok(connector.connect(domain, stream).await?)
}
