// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! Proxy to use Nostr Browser signer ([NIP-07](https://github.com/nostr-protocol/nips/blob/master/07.md)) in native applications.
//!
//! <https://github.com/nostr-protocol/nips/blob/master/07.md>

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::large_futures)]
#![warn(rustdoc::bare_urls)]

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use atomic_destructor::{AtomicDestroyer, AtomicDestructor};
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use nostr::prelude::{BoxedFuture, SignerBackend};
use nostr::{Event, NostrSigner, PublicKey, SignerError, UnsignedEvent};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot::Sender;
use tokio::sync::{Mutex, Notify, oneshot};
use tokio::time;
use uuid::Uuid;

mod error;
pub mod prelude;

pub use self::error::Error;

const HTML: &str = include_str!("../index.html");
const JS: &str = include_str!("../proxy.js");
const CSS: &str = include_str!("../style.css");

type PendingResponseMap = HashMap<Uuid, Sender<Result<Value, String>>>;

#[derive(Debug, Deserialize)]
struct Message {
    id: Uuid,
    error: Option<String>,
    result: Option<Value>,
}

impl Message {
    fn into_result(self) -> Result<Value, String> {
        if let Some(error) = self.error {
            Err(error)
        } else {
            Ok(self.result.unwrap_or(Value::Null))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum RequestMethod {
    GetPublicKey,
    SignEvent,
    Nip04Encrypt,
    Nip04Decrypt,
    Nip44Encrypt,
    Nip44Decrypt,
}

impl RequestMethod {
    fn as_str(&self) -> &str {
        match self {
            Self::GetPublicKey => "get_public_key",
            Self::SignEvent => "sign_event",
            Self::Nip04Encrypt => "nip04_encrypt",
            Self::Nip04Decrypt => "nip04_decrypt",
            Self::Nip44Encrypt => "nip44_encrypt",
            Self::Nip44Decrypt => "nip44_decrypt",
        }
    }
}

impl Serialize for RequestMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
struct RequestData {
    id: Uuid,
    method: RequestMethod,
    params: Value,
}

impl RequestData {
    #[inline]
    fn new(method: RequestMethod, params: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            method,
            params,
        }
    }
}

#[derive(Serialize)]
struct Requests<'a> {
    requests: &'a [RequestData],
}

impl<'a> Requests<'a> {
    #[inline]
    fn new(requests: &'a [RequestData]) -> Self {
        Self { requests }
    }

    #[inline]
    fn len(&self) -> usize {
        self.requests.len()
    }
}

/// Params for NIP-04 and NIP-44 encryption/decryption
#[derive(Serialize)]
struct CryptoParams<'a> {
    public_key: &'a PublicKey,
    content: &'a str,
}

impl<'a> CryptoParams<'a> {
    #[inline]
    fn new(public_key: &'a PublicKey, content: &'a str) -> Self {
        Self {
            public_key,
            content,
        }
    }
}

#[derive(Debug)]
struct ProxyState {
    /// Requests waiting to be picked up by browser
    pub outgoing_requests: Mutex<Vec<RequestData>>,
    /// Map of request ID to response sender
    pub pending_responses: Mutex<PendingResponseMap>,
    /// Last time the client ask for the pending requests
    pub last_pending_request: Arc<AtomicU64>,
}

/// Configuration options for [`BrowserSignerProxy`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSignerProxyOptions {
    /// Request timeout for the signer extension. Default is 30 seconds.
    pub timeout: Duration,
    /// Proxy server IP address and port. Default is `127.0.0.1:7400`.
    pub addr: SocketAddr,
}

#[derive(Debug, Clone)]
struct InnerBrowserSignerProxy {
    /// Configuration options for the proxy
    options: BrowserSignerProxyOptions,
    /// Internal state of the proxy including request queues
    state: Arc<ProxyState>,
    /// Notification trigger for graceful shutdown
    shutdown: Arc<Notify>,
    /// Flag to indicate if the server is shutdown
    is_shutdown: Arc<AtomicBool>,
    /// Flat indicating if the server is started
    is_started: Arc<AtomicBool>,
}

impl AtomicDestroyer for InnerBrowserSignerProxy {
    fn on_destroy(&self) {
        self.shutdown();
    }
}

impl InnerBrowserSignerProxy {
    #[inline]
    fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }

    fn shutdown(&self) {
        // Mark the server as shutdown
        self.is_shutdown.store(true, Ordering::SeqCst);

        // Notify all waiters that the proxy is shutting down
        self.shutdown.notify_one();
        self.shutdown.notify_waiters();
    }
}

/// Nostr Browser Signer Proxy
///
/// Proxy to use Nostr Browser signer (NIP-07) in native applications.
#[derive(Debug, Clone)]
pub struct BrowserSignerProxy {
    inner: AtomicDestructor<InnerBrowserSignerProxy>,
}

impl Default for BrowserSignerProxyOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            // 7 for NIP-07 and 400 because the NIP title is 40 bytes :)
            addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7400)),
        }
    }
}

impl BrowserSignerProxyOptions {
    /// Sets the timeout duration.
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the IP address.
    pub const fn ip_addr(mut self, new_ip: IpAddr) -> Self {
        self.addr = SocketAddr::new(new_ip, self.addr.port());
        self
    }

    /// Sets the port number.
    pub const fn port(mut self, new_port: u16) -> Self {
        self.addr = SocketAddr::new(self.addr.ip(), new_port);
        self
    }
}

impl BrowserSignerProxy {
    /// Construct a new browser signer proxy
    pub fn new(options: BrowserSignerProxyOptions) -> Self {
        let state = ProxyState {
            outgoing_requests: Mutex::new(Vec::new()),
            pending_responses: Mutex::new(HashMap::new()),
            last_pending_request: Arc::new(AtomicU64::new(0)),
        };

        Self {
            inner: AtomicDestructor::new(InnerBrowserSignerProxy {
                options,
                state: Arc::new(state),
                shutdown: Arc::new(Notify::new()),
                is_shutdown: Arc::new(AtomicBool::new(false)),
                is_started: Arc::new(AtomicBool::new(false)),
            }),
        }
    }

    /// Indicates whether the server is currently running.
    #[inline]
    pub fn is_started(&self) -> bool {
        self.inner.is_started.load(Ordering::SeqCst)
    }

    /// Checks if there is an open browser tap ready to respond to requests by
    /// verifying the time since the last pending request.
    #[inline]
    pub fn is_session_active(&self) -> bool {
        current_time() - self.inner.state.last_pending_request.load(Ordering::SeqCst) < 2
    }

    /// Get the signer proxy webpage URL
    #[inline]
    pub fn url(&self) -> String {
        format!("http://{}", self.inner.options.addr)
    }

    /// Start the proxy
    ///
    /// If this is not called, will be automatically started on the first interaction with the signer.
    pub async fn start(&self) -> Result<(), Error> {
        // Ensure is not shutdown
        if self.inner.is_shutdown() {
            return Err(Error::Shutdown);
        }

        // Mark the proxy as started and check if was already started
        let is_started: bool = self.inner.is_started.swap(true, Ordering::SeqCst);

        // Immediately return if already started
        if is_started {
            return Ok(());
        }

        let listener: TcpListener = match TcpListener::bind(self.inner.options.addr).await {
            Ok(listener) => listener,
            Err(e) => {
                // Undo the started flag if binding fails
                self.inner.is_started.store(false, Ordering::SeqCst);

                // Propagate error
                return Err(Error::from(e));
            }
        };

        let addr: SocketAddr = self.inner.options.addr;
        let state: Arc<ProxyState> = self.inner.state.clone();
        let shutdown: Arc<Notify> = self.inner.shutdown.clone();

        tokio::spawn(async move {
            tracing::info!("Starting proxy server on {addr}");

            loop {
                tokio::select! {
                    res = listener.accept() => {
                        let stream: TcpStream = match res {
                            Ok((stream, ..)) => stream,
                            Err(e) => {
                                tracing::error!("Failed to accept connection: {}", e);
                                continue;
                            }
                        };

                        let io: TokioIo<TcpStream> = TokioIo::new(stream);
                        let state: Arc<ProxyState> = state.clone();
                        let shutdown: Arc<Notify> = shutdown.clone();

                        tokio::spawn(async move {
                            let service = service_fn(move |req| {
                                handle_request(req, state.clone())
                            });

                            tokio::select! {
                                res = http1::Builder::new().serve_connection(io, service) => {
                                    if let Err(e) = res {
                                        tracing::error!("Error serving connection: {e}");
                                    }
                                }
                                _ = shutdown.notified() => {
                                        tracing::debug!("Closing connection, proxy server is shutting down.");
                                    }
                                }
                        });
                    },
                    _ = shutdown.notified() => {
                        break;
                    }
                }
            }

            tracing::info!("Shutting down proxy server.");
        });

        Ok(())
    }

    #[inline]
    async fn store_pending_response(&self, id: Uuid, tx: Sender<Result<Value, String>>) {
        let mut pending_responses = self.inner.state.pending_responses.lock().await;
        pending_responses.insert(id, tx);
    }

    #[inline]
    async fn store_outgoing_request(&self, request: RequestData) {
        let mut outgoing_requests = self.inner.state.outgoing_requests.lock().await;
        outgoing_requests.push(request);
    }

    async fn request<T>(&self, method: RequestMethod, params: Value) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        // Start the proxy if not already started
        self.start().await?;

        // Construct the request
        let request: RequestData = RequestData::new(method, params);

        // Create a oneshot channel
        let (tx, rx) = oneshot::channel();

        // Store the response sender
        self.store_pending_response(request.id, tx).await;

        // Add to outgoing requests queue
        self.store_outgoing_request(request).await;

        // Wait for response
        match time::timeout(self.inner.options.timeout, rx)
            .await
            .map_err(|_| Error::Timeout)??
        {
            Ok(res) => Ok(serde_json::from_value(res)?),
            Err(error) => Err(Error::Generic(error)),
        }
    }

    #[inline]
    async fn _get_public_key(&self) -> Result<PublicKey, Error> {
        self.request(RequestMethod::GetPublicKey, json!({})).await
    }

    #[inline]
    async fn _sign_event(&self, event: UnsignedEvent) -> Result<Event, Error> {
        let event: Event = self
            .request(RequestMethod::SignEvent, serde_json::to_value(event)?)
            .await?;
        event.verify()?;
        Ok(event)
    }

    #[inline]
    async fn _nip04_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String, Error> {
        let params = CryptoParams::new(public_key, content);
        self.request(RequestMethod::Nip04Encrypt, serde_json::to_value(params)?)
            .await
    }

    #[inline]
    async fn _nip04_decrypt(&self, public_key: &PublicKey, content: &str) -> Result<String, Error> {
        let params = CryptoParams::new(public_key, content);
        self.request(RequestMethod::Nip04Decrypt, serde_json::to_value(params)?)
            .await
    }

    #[inline]
    async fn _nip44_encrypt(&self, public_key: &PublicKey, content: &str) -> Result<String, Error> {
        let params = CryptoParams::new(public_key, content);
        self.request(RequestMethod::Nip44Encrypt, serde_json::to_value(params)?)
            .await
    }

    #[inline]
    async fn _nip44_decrypt(&self, public_key: &PublicKey, content: &str) -> Result<String, Error> {
        let params = CryptoParams::new(public_key, content);
        self.request(RequestMethod::Nip44Decrypt, serde_json::to_value(params)?)
            .await
    }
}

impl NostrSigner for BrowserSignerProxy {
    fn backend(&self) -> SignerBackend {
        SignerBackend::BrowserExtension
    }

    #[inline]
    fn get_public_key(&self) -> BoxedFuture<Result<PublicKey, SignerError>> {
        Box::pin(async move { self._get_public_key().await.map_err(SignerError::backend) })
    }

    #[inline]
    fn sign_event(&self, unsigned: UnsignedEvent) -> BoxedFuture<Result<Event, SignerError>> {
        Box::pin(async move {
            self._sign_event(unsigned)
                .await
                .map_err(SignerError::backend)
        })
    }

    #[inline]
    fn nip04_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self._nip04_encrypt(public_key, content)
                .await
                .map_err(SignerError::backend)
        })
    }

    #[inline]
    fn nip04_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        encrypted_content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self._nip04_decrypt(public_key, encrypted_content)
                .await
                .map_err(SignerError::backend)
        })
    }

    #[inline]
    fn nip44_encrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        content: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self._nip44_encrypt(public_key, content)
                .await
                .map_err(SignerError::backend)
        })
    }

    #[inline]
    fn nip44_decrypt<'a>(
        &'a self,
        public_key: &'a PublicKey,
        payload: &'a str,
    ) -> BoxedFuture<'a, Result<String, SignerError>> {
        Box::pin(async move {
            self._nip44_decrypt(public_key, payload)
                .await
                .map_err(SignerError::backend)
        })
    }
}

async fn handle_request(
    req: Request<Incoming>,
    state: Arc<ProxyState>,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    match (req.method(), req.uri().path()) {
        // Serve the HTML proxy page
        (&Method::GET, "/") => Ok(Response::builder()
            .header("Content-Type", "text/html")
            .body(full(HTML))?),
        // Serve the CSS page style
        (&Method::GET, "/style.css") => Ok(Response::builder()
            .header("Content-Type", "text/css")
            .body(full(CSS))?),
        // Serve the JS proxy script
        (&Method::GET, "/proxy.js") => Ok(Response::builder()
            .header("Content-Type", "application/javascript")
            .body(full(JS))?),
        // Browser polls this endpoint to get pending requests
        (&Method::GET, "/api/pending") => {
            state
                .last_pending_request
                .store(current_time(), Ordering::SeqCst);

            let mut outgoing = state.outgoing_requests.lock().await;

            let requests: Requests<'_> = Requests::new(&outgoing);
            let json: String = serde_json::to_string(&requests)?;

            tracing::debug!("Sending {} pending requests to browser", requests.len());

            // Clear the outgoing requests after sending them
            outgoing.clear();

            Ok(Response::builder()
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(full(json))?)
        }
        // Get response
        (&Method::POST, "/api/response") => {
            // Correctly collect the body bytes from the stream
            let body_bytes: Bytes = match req.into_body().collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(e) => {
                    tracing::error!("Failed to read body: {e}");
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("Failed to read body"))?;
                    return Ok(response);
                }
            };

            // Handle responses from the browser extension
            let message: Message = match serde_json::from_slice(&body_bytes) {
                Ok(json) => json,
                Err(_) => {
                    let response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("Invalid JSON"))?;
                    return Ok(response);
                }
            };

            tracing::debug!("Received response from browser: {message:?}");

            let id: Uuid = message.id;
            let mut pending = state.pending_responses.lock().await;

            match pending.remove(&id) {
                Some(sender) => {
                    let _ = sender.send(message.into_result());
                    tracing::info!("Forwarded response for request {id}");
                }
                None => tracing::warn!("No pending request found for {id}"),
            }

            let response = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .body(full("OK"))?;
            Ok(response)
        }
        (&Method::OPTIONS, _) => {
            // Handle CORS preflight requests
            let response = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .body(full(""))?;
            Ok(response)
        }
        // 404 - not found
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full("Not Found"))?;
            Ok(response)
        }
    }
}

#[inline]
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/// Gets the current time in seconds since the Unix epoch (1970-01-01). If the
/// time is before the epoch, returns 0.
#[inline]
fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
