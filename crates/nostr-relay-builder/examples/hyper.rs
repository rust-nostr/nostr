// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use base64::prelude::*;
use hyper::body::Incoming;
use hyper::header::{CONNECTION, SEC_WEBSOCKET_ACCEPT, UPGRADE};
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use nostr::hashes::sha1::Hash as Sha1Hash;
use nostr::hashes::{Hash, HashEngine};
use nostr_relay_builder::{LocalRelay, RelayBuilder};
use tokio::net::TcpListener;

struct HttpServer {
    relay: LocalRelay,
    remote: SocketAddr,
}

/// Copied from https://github.com/snapview/tungstenite-rs/blob/c16778797b2eeb118aa064aa5b483f90c3989627/src/handshake/mod.rs#L112C1-L125C1
/// Derive the `Sec-WebSocket-Accept` response header from a `Sec-WebSocket-Key` request header.
///
/// This function can be used to perform a handshake before passing a raw TCP stream to
/// [`WebSocket::from_raw_socket`][crate::protocol::WebSocket::from_raw_socket].
pub fn derive_accept_key(request_key: &[u8]) -> String {
    // ... field is constructed by concatenating /key/ ...
    // ... with the string "258EAFA5-E914-47DA-95CA-C5AB0DC85B11" (RFC 6455)
    const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut engine = Sha1Hash::engine();
    engine.input(request_key);
    engine.input(WS_GUID);
    let hash: Sha1Hash = Sha1Hash::from_engine(engine);
    BASE64_STANDARD.encode(hash)
}

impl HttpServer {
    fn new(relay: LocalRelay, remote: SocketAddr) -> Self {
        HttpServer { relay, remote }
    }
}

impl Service<Request<Incoming>> for HttpServer {
    type Response = Response<String>;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let base = Response::builder()
            .header("server", "nostr-relay-builder")
            .status(404);

        // check is upgrade
        if let (Some(c), Some(w)) = (
            req.headers().get("connection"),
            req.headers().get("upgrade"),
        ) {
            if c.to_str()
                .map(|s| s.to_lowercase() == "upgrade")
                .unwrap_or(false)
                && w.to_str()
                    .map(|s| s.to_lowercase() == "websocket")
                    .unwrap_or(false)
            {
                let key = req.headers().get("sec-websocket-key");
                let derived = key.map(|k| derive_accept_key(k.as_bytes()));

                let addr = self.remote;
                let relay = self.relay.clone();
                tokio::spawn(async move {
                    match hyper::upgrade::on(req).await {
                        Ok(upgraded) => {
                            if let Err(e) =
                                relay.take_connection(TokioIo::new(upgraded), addr).await
                            {
                                tracing::error!("{}", e);
                            }
                        }
                        Err(e) => tracing::error!("{}", e),
                    }
                });
                return Box::pin(async move {
                    Ok(base
                        .status(101)
                        .header(CONNECTION, "upgrade")
                        .header(UPGRADE, "websocket")
                        .header(SEC_WEBSOCKET_ACCEPT, derived.unwrap())
                        .body("".to_string())
                        .unwrap())
                });
            }
        }

        // serve landing page otherwise
        Box::pin(async move {
            Ok(base
                .status(200)
                .header("content-type", "text/html")
                .body(
                    "<html><body><h1>Welcome to nostr-relay-builder-hyper</h1></body></html>"
                        .to_string(),
                )
                .unwrap())
        })
    }
}

#[tokio::main]
async fn main() -> nostr_relay_builder::prelude::Result<()> {
    tracing_subscriber::fmt::init();

    let builder = RelayBuilder::default();
    let relay = LocalRelay::new(builder).await?;

    let http_addr: SocketAddr = "127.0.0.1:8000".parse()?;
    let listener = TcpListener::bind(&http_addr).await?;
    loop {
        let (socket, addr) = listener.accept().await?;

        let io = TokioIo::new(socket);
        let server = HttpServer::new(relay.clone(), addr);
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, server)
                .with_upgrades()
                .await
            {
                tracing::error!("Failed to handle request: {}", e);
            }
        });
    }
}
